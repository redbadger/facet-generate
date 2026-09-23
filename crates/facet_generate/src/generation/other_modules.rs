//! What a module being generated knows about the types of the *other* modules
//! of the same registry.
//!
//! The installers split a registry into one module per namespace, and each
//! module's [`CodeGeneratorConfig`](super::CodeGeneratorConfig) is filled from
//! that module's types alone. How a plugin serializes a reference to a type
//! can depend on what kind of type it is, though: a TypeScript enum goes
//! through its standalone `serializeX` / `deserializeX` functions, and a C#
//! all-unit enum through its `XBincode` helper class. So a reference to an
//! enum in another namespace needs to know that it is one (#154).
//!
//! The installers hand the whole registry to the generator, which records the
//! kind of every type from another module here, under the name its references
//! are rewritten to, for as long as it emits the module. The plugins only see
//! the module's config, so this is a thread-local rather than a field on it,
//! which keeps the public API and the config unchanged. Outside that scope,
//! and for every type the module declares itself, [`kind`] returns `None` and
//! the plugins decide from the config as before.

use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

use crate::{
    Registry,
    reflection::format::{ContainerFormat, QualifiedTypeName, VariantFormat},
};

/// What kind of type a container is, as far as serializing a reference to it
/// is concerned.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    /// A struct of any shape.
    Struct,
    /// An enum with at least one variant that carries data.
    DataEnum,
    /// An enum whose variants are all unit variants.
    UnitEnum,
}

impl Kind {
    fn of(format: &ContainerFormat) -> Self {
        match format {
            ContainerFormat::Enum(variants, _, _)
                if variants
                    .values()
                    .all(|v| matches!(v.value, VariantFormat::Unit)) =>
            {
                Self::UnitEnum
            }
            ContainerFormat::Enum(..) => Self::DataEnum,
            _ => Self::Struct,
        }
    }

    #[cfg(feature = "typescript")]
    pub(crate) const fn is_enum(self) -> bool {
        matches!(self, Self::DataEnum | Self::UnitEnum)
    }

    #[cfg(feature = "csharp")]
    pub(crate) const fn is_unit_enum(self) -> bool {
        matches!(self, Self::UnitEnum)
    }
}

/// The types of the other modules of a registry, keyed by the name a
/// reference to each is rewritten to in the module being generated.
#[derive(Debug, Default)]
pub(crate) struct OtherModules(BTreeMap<QualifiedTypeName, Kind>);

impl OtherModules {
    /// Indexes the types of `whole` that are not in `local` (the module being
    /// generated), respelled with `requalify`, the function the generator
    /// rewrites each type reference in `local` with.
    ///
    /// A respelled name that is also the respelling of one of the module's own
    /// types means the module's own type, so it is left out. So is a name two
    /// other types respell to, since it is ambiguous. [`kind`] then returns
    /// `None` for both, and the plugins fall back to the config.
    pub(crate) fn new(
        whole: &Registry,
        local: &Registry,
        requalify: impl Fn(&QualifiedTypeName) -> QualifiedTypeName,
    ) -> Self {
        let own: BTreeSet<QualifiedTypeName> = local.keys().map(&requalify).collect();

        let mut types = BTreeMap::new();
        let mut ambiguous = BTreeSet::new();
        for (name, format) in whole {
            if local.contains_key(name) {
                continue;
            }
            let name = requalify(name);
            if own.contains(&name) {
                continue;
            }
            if types.insert(name.clone(), Kind::of(format)).is_some() {
                ambiguous.insert(name);
            }
        }
        for name in &ambiguous {
            types.remove(name);
        }

        Self(types)
    }

    /// Runs `f` with `other_modules` as the answer to [`kind`], then restores
    /// whatever was there before (also if `f` panics). With `None`, `kind`
    /// returns `None` throughout `f`.
    pub(crate) fn scope<R>(other_modules: Option<Self>, f: impl FnOnce() -> R) -> R {
        struct Restore(Option<Rc<OtherModules>>);

        impl Drop for Restore {
            fn drop(&mut self) {
                let previous = self.0.take();
                CURRENT.with(|current| *current.borrow_mut() = previous);
            }
        }

        let previous = CURRENT.with(|current| current.replace(other_modules.map(Rc::new)));
        let _restore = Restore(previous);
        f()
    }
}

thread_local! {
    static CURRENT: RefCell<Option<Rc<OtherModules>>> = const { RefCell::new(None) };
}

/// The kind of the type from another module that a reference spelled `name`
/// (as the emitter sees it) points at, while a generator is emitting a module
/// with the whole registry in hand.
///
/// `None` if `name` is not a type from another module, or if no generator is
/// emitting one with the whole registry (for example when a generator is used
/// on its own, without an installer).
pub(crate) fn kind(name: &QualifiedTypeName) -> Option<Kind> {
    CURRENT.with(|current| {
        current
            .borrow()
            .as_ref()
            .and_then(|other_modules| other_modules.0.get(name).copied())
    })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::reflection::format::{Doc, EnumTagging, Named, Namespace};

    fn unit_enum() -> ContainerFormat {
        let mut variants = BTreeMap::new();
        variants.insert(0u32, Named::new(&VariantFormat::Unit, "On".to_string()));
        ContainerFormat::Enum(variants, EnumTagging::External, Doc::default())
    }

    /// The TypeScript rewrite: a type in the module being generated (`kit`)
    /// is written bare, like a ROOT one.
    fn strip_kit(name: &QualifiedTypeName) -> QualifiedTypeName {
        match &name.namespace {
            Namespace::Named(namespace) if namespace == "kit" => {
                QualifiedTypeName::root(name.name.clone())
            }
            _ => name.clone(),
        }
    }

    #[test]
    fn knows_the_types_of_the_other_modules_only_while_in_scope() {
        let kit_presence = QualifiedTypeName::namespaced("kit".to_string(), "Presence".to_string());
        let root_card = QualifiedTypeName::root("Card".to_string());

        let whole = Registry::from([
            (kit_presence.clone(), unit_enum()),
            (
                root_card.clone(),
                ContainerFormat::UnitStruct(Doc::default()),
            ),
        ]);
        let local = Registry::from([(
            root_card.clone(),
            ContainerFormat::UnitStruct(Doc::default()),
        )]);

        assert_eq!(kind(&kit_presence), None);
        OtherModules::scope(
            Some(OtherModules::new(&whole, &local, Clone::clone)),
            || {
                assert_eq!(kind(&kit_presence), Some(Kind::UnitEnum));
                // The module's own types are left to the config.
                assert_eq!(kind(&root_card), None);
                OtherModules::scope(None, || assert_eq!(kind(&kit_presence), None));
                assert_eq!(kind(&kit_presence), Some(Kind::UnitEnum));
            },
        );
        assert_eq!(kind(&kit_presence), None);
    }

    #[test]
    fn the_module_s_own_types_win_a_respelled_name() {
        let kit_presence = QualifiedTypeName::namespaced("kit".to_string(), "Presence".to_string());
        let root_presence = QualifiedTypeName::root("Presence".to_string());

        // A ROOT enum, and a struct of the same name in `kit`, whose
        // references are respelled bare like ROOT ones.
        let whole = Registry::from([
            (root_presence.clone(), unit_enum()),
            (
                kit_presence.clone(),
                ContainerFormat::UnitStruct(Doc::default()),
            ),
        ]);
        let local = Registry::from([(kit_presence, ContainerFormat::UnitStruct(Doc::default()))]);

        OtherModules::scope(Some(OtherModules::new(&whole, &local, strip_kit)), || {
            assert_eq!(kind(&root_presence), None);
        });
    }
}
