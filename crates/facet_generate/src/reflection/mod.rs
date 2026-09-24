//! Type reflection engine — extracts Rust type metadata into a language-neutral [`Registry`].
//!
//! Uses the [`facet`] crate's compile-time `Shape` descriptors to walk type graphs recursively.
//! Each struct or enum becomes a [`ContainerFormat`] entry in the registry. The shape of each
//! field, variant, or nested value is described by a [`Format`].
//!
//! A "format" is an AST node describing a type's serialization shape — how it would be laid out
//! on the wire. [`ContainerFormat`] describes named, top-level types (unit structs, newtype
//! structs, structs with fields, and enums with variants). Each container is composed of
//! [`Format`] nodes, which describe the value types within: primitives, options, sequences,
//! maps, tuples, and references to other containers via `Format::TypeName`.
//!
//! The main entry point is [`RegistryBuilder`], which provides a builder-pattern API:
//!
//! ```ignore
//! let registry = RegistryBuilder::new()
//!     .add_type::<MyStruct>()?
//!     .add_type::<MyEnum>()?
//!     .build()?;
//! ```
//!
//! Key responsibilities:
//! - Mapping `facet` shapes (structs, enums, primitives, sequences, maps, pointers) to `Format` AST nodes
//! - Resolving generic types (`Option`, `Vec`, `HashMap`, `Arc`, `Box`, etc.) into their format equivalents
//! - Handling transparent wrappers and newtypes
//! - Propagating and resolving namespace annotations via a context stack
//! - Detecting conflicts: a generic type instantiated with different type parameters, or two
//!   different Rust types that would generate the same name in the same namespace

pub mod format;
#[cfg(test)]
pub mod regression_tests;

use std::{
    collections::{BTreeMap, HashMap},
    string::ToString,
    sync::LazyLock,
};

use facet::{
    ArrayDef, ConstTypeId, DeclId, Def, EnumType, Facet, Field, FieldFlags, NumericType,
    PointerDef, PointerType, PrimitiveType, Shape, StructKind, StructType, TextualType, Type,
    UserType, Variant,
};
use regex::Regex;

use crate as fg;
use crate::{Registry, error::Error};

use format::{
    ContainerFormat, EnumTagging, Format, FormatHolder, Named, Namespace, QualifiedTypeName,
    VariantFormat,
};

const SUPPORTED_GENERIC_TYPES: [&str; 10] = [
    "Arc", "Rc", "Box", "Option", "Vec", "HashMap", "HashSet", "BTreeMap", "BTreeSet", "DateTime",
];

/// The Rust type that owns a generated name.
///
/// Two different Rust types can generate the same [`QualifiedTypeName`], so the identity of the
/// type that got there first is kept alongside the name, both to recognise recursion back into
/// the same type and to name the offender when a second type claims the name.
#[derive(Debug)]
struct Claim {
    id: ConstTypeId,
    rust_path: String,
}

/// A recorded rename, together with the Rust type it applies to.
///
/// The key is the unrenamed name, which another type of the same name would also produce, so the
/// mapping is only applied to the type that registered it.
#[derive(Debug)]
struct Mapping {
    id: ConstTypeId,
    renamed: QualifiedTypeName,
}

/// The Rust path of a type, for use in error messages.
///
/// Derived types carry their module path; primitives and foreign types do not, and are named by
/// their `Display` alone.
fn rust_path(shape: &Shape) -> String {
    shape
        .module_path
        .map_or_else(|| shape.to_string(), |path| format!("{path}::{shape}"))
}

/// The type a shape is generated as.
///
/// A transparent wrapper is generated as the type it wraps, under that type's name, so the two
/// share an identity here — however long the chain of wrappers. Any other shape is its own.
fn generated_shape(mut shape: &Shape) -> &Shape {
    while is_transparent_shape(shape)
        && let Some(inner) = shape.inner
    {
        shape = inner;
    }
    shape
}

/// The type a chain of transparent wrappers is generated as, with the namespace context that the
/// innermost wrapper with an `fg::namespace` of its own sets for it, if any.
///
/// A shape that is not a transparent wrapper is returned as it is, with no context.
fn strip_transparent(mut shape: &Shape) -> Result<(&Shape, Option<NamespaceContext>), Error> {
    let mut context = None;
    while is_transparent_shape(shape)
        && let Some(inner) = shape.inner
    {
        if let NamespaceAction::SetContext(wrapper) = extract_namespace_from_shape(shape)? {
            context = Some(wrapper);
        }
        shape = inner;
    }
    Ok((shape, context))
}

/// A namespace context with its source information
#[derive(Debug, Clone, PartialEq)]
struct NamespaceContext {
    namespace: Namespace,
    explicit: bool, // true = explicitly set, false = inherited
}

impl NamespaceContext {
    /// Creates a cleared namespace context (explicit root)
    const fn cleared() -> Self {
        Self {
            namespace: Namespace::Root,
            explicit: true,
        }
    }

    /// Creates an explicitly set namespace context
    const fn explicit(namespace: Namespace) -> Self {
        Self {
            namespace,
            explicit: true,
        }
    }

    /// Checks if this context is explicit
    const fn is_explicit(&self) -> bool {
        self.explicit
    }

    /// Checks if this context was cleared (explicit root)
    const fn is_cleared(&self) -> bool {
        self.explicit && matches!(self.namespace, Namespace::Root)
    }
}

/// Action to take with namespace context
#[derive(Debug, Clone, PartialEq)]
enum NamespaceAction {
    /// Set a specific namespace context
    SetContext(NamespaceContext),
    /// Inherit the current context unchanged (push current context again)
    Inherit,
}

impl NamespaceAction {
    /// Returns true if this action represents an explicit namespace setting
    const fn is_explicit(&self) -> bool {
        match self {
            Self::SetContext(ctx) => ctx.is_explicit(),
            Self::Inherit => false, // Inheriting is not explicit
        }
    }
}

/// The bridge between `facet` compile-time type metadata and the [`Registry`] consumed by
/// code generators.
///
/// Types are added with [`add_type`](Self::add_type), which recursively reflects the type and
/// all types reachable from its fields and variants. The builder tracks which types have already
/// been processed to avoid duplicates, and detects conflicts: a generic type instantiated with
/// different type parameters, or two different Rust types that would generate the same name in
/// the same namespace.
#[derive(Debug, Default)]
pub struct RegistryBuilder {
    pub registry: Registry,
    current: Vec<QualifiedTypeName>,
    processed: HashMap<QualifiedTypeName, Claim>,
    name_mappings: BTreeMap<QualifiedTypeName, Mapping>,
    generic_type_params: BTreeMap<DeclId, String>,
    namespace_context_stack: Vec<NamespaceContext>,
    type_namespace_sources: HashMap<QualifiedTypeName, bool>, // true = explicit, false = inherited
}

impl RegistryBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds the registry from the current state.
    /// # Errors
    /// Will return an error with a suitable error message if the registry is invalid,
    /// usually due to incomplete reflection.
    pub fn build(self) -> Result<Registry, Error> {
        for (type_name, format) in &self.registry {
            if let Err(err) = format.visit(&mut |_| Ok(())) {
                return Err(Error::ReflectionError {
                    type_name: type_name.clone().to_string(),
                    message: err.to_string(),
                });
            }
        }

        check_references(&self.registry)?;

        Ok(self.registry)
    }

    /// Reflect a type into the registry.
    ///
    /// # Errors
    /// Will return an error if:
    /// * there is a problem building the registry,
    /// * non-special generic types are used with different type parameters,
    /// * there is an unsupported layout,
    /// * namespaces are conflicting,
    /// * namespaces have invalid names, or
    /// * attributes are malformed.
    pub fn add_type<'a, T: Facet<'a>>(mut self) -> Result<Self, Error> {
        self.register_reachable(T::SHAPE)?;
        Ok(self)
    }

    /// The [`Format`] that a struct field of type `T` would be given.
    ///
    /// This is the reflection-level answer to "how do I refer to `T` from
    /// generated code?", and is intended for [`EmitterPlugin`] authors who
    /// need to emit a type reference or a serialization call for a type that
    /// is not itself the container being emitted (for example the payload
    /// type of a generated method).
    ///
    /// - A named container (struct or enum) becomes
    ///   [`Format::TypeName`] carrying its [`QualifiedTypeName`], honouring
    ///   `#[facet(rename = "…")]`, `#[facet(fg::namespace = "…")]` and
    ///   `#[facet(transparent)]` exactly as the registry does.
    /// - `()` becomes [`Format::Unit`].
    /// - `Option<T>`, `Vec<T>`, `HashMap<K, V>`, `HashSet<T>`, `[T; N]`,
    ///   tuples, smart pointers and primitives become the corresponding
    ///   structural [`Format`], recursively.
    ///
    /// `T`'s container types should have been added with
    /// [`add_type`](Self::add_type) first: renames applied while reflecting a
    /// container are recorded on the builder, and a type this builder has
    /// never seen is named from its own attributes alone.
    ///
    /// Field-level attributes have no analogue here — `format_of::<Vec<u8>>()`
    /// is `Seq(U8)`, not `Bytes`, because `#[facet(fg::bytes)]` is a property
    /// of the field, not of the type.
    ///
    /// The type names in the result are in registry spelling, as the registry
    /// keys are. The emitters rewrite every reference before emitting it, so
    /// requalify the format once with the target language's `requalify_format`
    /// (for example
    /// [`csharp::requalify_format`](crate::generation::csharp::requalify_format))
    /// before passing it to the language's `render_type` or
    /// `write_serialize_value`.
    ///
    /// # Errors
    ///
    /// Returns an error if `T` (or a type reachable from it) cannot be
    /// reflected — for example an unsupported scalar.
    ///
    /// [`EmitterPlugin`]: crate::generation::plugin::EmitterPlugin
    pub fn format_of<'a, T: Facet<'a>>(&self) -> Result<Format, Error> {
        self.format_of_shape(T::SHAPE)
    }

    /// Read-only counterpart of [`Self::reference_to`], used by [`format_of`](Self::format_of).
    fn format_of_shape(&self, shape: &Shape) -> Result<Format, Error> {
        reference_format(shape, None, &mut |shape, wrapper| {
            self.mapped_name(shape, wrapper)
        })
    }

    /// Read-only counterpart of [`Self::get_name_with_mappings`].
    ///
    /// Namespace *contexts* are a property of an in-progress walk, and the
    /// stack is empty once the types have been added, so only the recorded
    /// rename mappings, the type's own attributes and the namespace of a
    /// transparent wrapper around it (`wrapper`) contribute.
    fn mapped_name(
        &self,
        shape: &Shape,
        wrapper: Option<&NamespaceContext>,
    ) -> Result<QualifiedTypeName, Error> {
        let base_key = QualifiedTypeName::root(shape.type_identifier.to_string());
        if let Some(mapping) = self.name_mappings.get(&base_key)
            && mapping.id == generated_shape(shape).id
        {
            return Ok(mapping.renamed.clone());
        }
        let name = get_name(shape)?;
        match wrapper {
            Some(context) if !extract_namespace_from_shape(shape)?.is_explicit() => {
                Ok(QualifiedTypeName {
                    namespace: context.namespace.clone(),
                    name: name.name,
                })
            }
            _ => Ok(name),
        }
    }
}

impl RegistryBuilder {
    fn push_with_type_check(
        &mut self,
        name: QualifiedTypeName,
        container: ContainerFormat,
        shape: &Shape,
    ) -> Result<(), Error> {
        let is_explicit = extract_namespace_from_shape(shape)?.is_explicit();

        // Check for conflicts: type name in multiple namespaces with mixed explicit/inherited sources
        self.check_namespace_ambiguity(&name, is_explicit)?;

        self.registry.insert(name.clone(), container);
        self.current.push(name);
        Ok(())
    }

    fn register_type_mapping(
        &mut self,
        original: QualifiedTypeName,
        renamed: QualifiedTypeName,
        shape: &Shape,
    ) {
        self.name_mappings.insert(
            original,
            Mapping {
                id: generated_shape(shape).id,
                renamed,
            },
        );
    }

    /// Claims a generated name for a Rust type.
    ///
    /// Returns `Ok(true)` if this type already holds the name — the recursion case, where the
    /// caller should stop rather than descend again — and `Ok(false)` once the claim is recorded
    /// for the first time. A different Rust type already holding the name is a conflict, because
    /// only one of them can be generated under it.
    fn claim(&mut self, name: &QualifiedTypeName, shape: &Shape) -> Result<bool, Error> {
        if self.is_claimed_by(name, shape)? {
            return Ok(true);
        }
        self.record_claim(name.clone(), shape);
        Ok(false)
    }

    /// Whether `shape` already holds `name`, erroring if a different Rust type does.
    fn is_claimed_by(&self, name: &QualifiedTypeName, shape: &Shape) -> Result<bool, Error> {
        let shape = generated_shape(shape);
        match self.processed.get(name) {
            None => Ok(false),
            Some(claim) if claim.id == shape.id => Ok(true),
            Some(claim) => Err(Error::DuplicateTypeName {
                name: name.name.clone(),
                namespace: name.namespace.to_string(),
                existing: claim.rust_path.clone(),
                new: rust_path(shape),
            }),
        }
    }

    fn record_claim(&mut self, name: QualifiedTypeName, shape: &Shape) {
        let shape = generated_shape(shape);
        self.processed.insert(
            name,
            Claim {
                id: shape.id,
                rust_path: rust_path(shape),
            },
        );
    }

    fn pop(&mut self) {
        self.current.pop();
    }

    fn get_mut(&mut self) -> Option<&mut ContainerFormat> {
        if let Some(name) = self.current.last() {
            self.registry.get_mut(name)
        } else {
            None
        }
    }

    /// Registers the struct or enum `shape`, and every container it reaches.
    fn format_container(&mut self, shape: &Shape) -> Result<(), Error> {
        if !self.is_supported_generic_type(shape) {
            return Err(Error::UnsupportedGenericType(shape.to_string()));
        }
        match &shape.ty {
            Type::User(UserType::Struct(struct_type)) => self.format_struct(struct_type, shape),
            Type::User(UserType::Enum(enum_type)) => self.format_enum(enum_type, shape),
            _ => unreachable!("only structs and enums are containers"),
        }
    }

    fn is_supported_generic_type(&mut self, shape: &Shape) -> bool {
        // Return true immediately for non-generic or special cases
        if shape.type_params.is_empty() {
            return true;
        }

        // Skip lifetimes and arrays
        if shape.type_identifier.starts_with('&') || shape.type_identifier.starts_with('[') {
            return true;
        }

        // Accept known generic types unconditionally
        if SUPPORTED_GENERIC_TYPES.contains(&shape.type_identifier) {
            return true;
        }

        // For other generics, verify all type parameters are consistent with previous uses
        let current_params = format!("{:?}", shape.type_params);
        let previous_params = self
            .generic_type_params
            .entry(shape.decl_id)
            .or_insert_with(|| current_params.clone());

        *previous_params == current_params
    }

    fn format_struct(&mut self, struct_type: &StructType, shape: &Shape) -> Result<(), Error> {
        let struct_name = self.get_name_with_mappings(shape)?;

        // Check if already processed using the full namespaced name. This is the recursion case:
        // the reference to the struct was made by `reference_to`, so there is nothing to update.
        if self.claim(&struct_name, shape)? {
            return Ok(());
        }

        // Register name mapping if it's different from original
        if struct_name.name != shape.type_identifier {
            let name = QualifiedTypeName {
                namespace: struct_name.namespace.clone(),
                name: shape.type_identifier.to_string(),
            };
            self.register_type_mapping(name, struct_name.clone(), shape);
        }

        // Extract namespace from this enum if it has one
        let type_level_namespace = extract_namespace_from_shape(shape)?;

        // Handle namespace context for the enum processing
        self.push_namespace(type_level_namespace);

        match struct_type.kind {
            StructKind::Unit => {
                self.push_with_type_check(
                    struct_name,
                    ContainerFormat::UnitStruct(shape.into()),
                    shape,
                )?;
                self.pop();
            }
            StructKind::TupleStruct => {
                if struct_type.fields.len() == 1 {
                    let field = struct_type.fields[0];
                    let field_shape = field.shape();

                    // A newtype struct; a transparent one never gets here, as it is registered as
                    // the type it wraps
                    let container = ContainerFormat::NewTypeStruct(Box::default(), shape.into());
                    self.push_with_type_check(struct_name, container, shape)?;

                    // Process the inner field
                    if !self.try_handle_bytes_attribute(&field) {
                        self.push_positional_field(field_shape)?;
                    }
                } else {
                    // Handle tuple struct with multiple fields
                    let container = ContainerFormat::TupleStruct(vec![], shape.into());
                    self.push_with_type_check(struct_name, container, shape)?;
                    for field in struct_type.fields {
                        let skip = field.flags.contains(FieldFlags::SKIP);
                        if skip {
                            continue;
                        }
                        if !self.try_handle_bytes_attribute(field) {
                            self.push_positional_field(field.shape())?;
                        }
                    }
                }
                self.pop();
            }
            StructKind::Struct => {
                let container = ContainerFormat::Struct(vec![], shape.into());
                self.push_with_type_check(struct_name, container, shape)?;
                for field in struct_type.fields {
                    let skip = field.flags.contains(FieldFlags::SKIP);
                    if skip {
                        continue;
                    }
                    self.handle_struct_field(field)?;
                }

                // If all fields were skipped, convert to UnitStruct to avoid empty data class issues
                if let Some(ContainerFormat::Struct(fields, doc)) = self.get_mut()
                    && fields.is_empty()
                {
                    let unit_container = ContainerFormat::UnitStruct(doc.clone());
                    if let Some(current_name) = self.current.last() {
                        self.registry.insert(current_name.clone(), unit_container);
                    }
                }

                self.pop();
            }
            StructKind::Tuple => unreachable!("anonymous tuples are not containers"),
        }

        self.pop_namespace();
        Ok(())
    }

    /// Gives the newtype or tuple struct being built its next field.
    ///
    /// A field whose type cannot be reflected is left out, as it always has been: a newtype keeps
    /// its unknown format, which `build` reports.
    fn push_positional_field(&mut self, field_shape: &Shape) -> Result<(), Error> {
        let Some(format) = self.field_format(field_shape)? else {
            return Ok(());
        };
        match self.get_mut() {
            Some(ContainerFormat::NewTypeStruct(inner_format, _doc)) => **inner_format = format,
            Some(ContainerFormat::TupleStruct(formats, _doc)) => formats.push(format),
            _ => {}
        }
        Ok(())
    }

    fn handle_struct_field(&mut self, field: &Field) -> Result<(), Error> {
        if self.try_handle_bytes_attribute(field) {
            return Ok(());
        }

        // Unlike a struct variant's, a struct field's own `fg::namespace` has never applied to an
        // `Option`, an anonymous tuple or a transparent wrapper, and still doesn't, so that the
        // output for those fields is unchanged.
        let field_shape = field.shape();
        let ignores_field_namespace = match &field_shape.ty {
            Type::User(UserType::Struct(struct_type)) => {
                struct_type.kind == StructKind::Tuple || is_transparent_shape(field_shape)
            }
            _ => matches!(field_shape.def, Def::Option(_)),
        };
        let format = if ignores_field_namespace {
            self.field_format(field_shape)?
        } else {
            self.named_field_format(field)?
        };

        if let Some(value) = format
            && let Some(ContainerFormat::Struct(named_formats, _doc)) = self.get_mut()
        {
            named_formats.push(Named {
                name: field_display_name(field),
                doc: field.into(),
                value,
            });
        }
        Ok(())
    }

    /// The format of a named field, of a struct or a struct variant, made under the field's own
    /// `fg::namespace` if it has one. See [`Self::field_format`].
    fn named_field_format(&mut self, field: &Field) -> Result<Option<Format>, Error> {
        let field_namespace = extract_namespace_from_field_attributes(field)?;
        self.push_namespace(field_namespace);
        let format = self.field_format(field.shape());
        self.pop_namespace();
        format
    }

    /// The format of a field of type `field_shape`, once the containers it reaches are registered,
    /// or `None` if the field is to be skipped (see [`Self::get_user_type_format`]).
    ///
    /// The reference and the registration are made under the same namespace context, so they
    /// agree, and the registration can't touch the container being built.
    fn field_format(&mut self, field_shape: &Shape) -> Result<Option<Format>, Error> {
        let Some(format) = self.get_user_type_format(field_shape)? else {
            return Ok(None);
        };
        self.process_nested_types(field_shape)?;
        Ok(Some(format))
    }

    fn try_handle_bytes_attribute(&mut self, field: &Field) -> bool {
        let Some(value) = bytes_attribute_format(field) else {
            return false;
        };
        let Some(container) = self.get_mut() else {
            return false;
        };
        match container {
            ContainerFormat::NewTypeStruct(format, _doc) => **format = value,
            ContainerFormat::TupleStruct(formats, _doc) => formats.push(value),
            ContainerFormat::Struct(nameds, _doc) => nameds.push(Named {
                name: field_display_name(field),
                doc: field.shape().into(),
                value,
            }),
            _ => return false,
        }
        true
    }

    fn format_enum(&mut self, enum_type: &EnumType, shape: &Shape) -> Result<(), Error> {
        let enum_name = self.get_name_with_mappings(shape)?;

        // Check if already processed using the full namespaced name
        if self.claim(&enum_name, shape)? {
            return Ok(());
        }

        // Register name mapping if it's different from original
        if enum_name.name != shape.type_identifier {
            let name = QualifiedTypeName {
                namespace: enum_name.namespace.clone(),
                name: shape.type_identifier.to_string(),
            };
            self.register_type_mapping(name, enum_name.clone(), shape);
        }

        // Extract namespace from this enum if it has one
        let type_level_namespace = extract_namespace_from_shape(shape)?;

        // Handle namespace context for the enum processing
        self.push_namespace(type_level_namespace);

        let variants = self.process_enum_variants(enum_type, shape)?;
        let container = ContainerFormat::Enum(variants, extract_enum_tagging(shape), shape.into());
        self.push_with_type_check(enum_name, container, shape)?;
        self.pop();

        self.pop_namespace();
        Ok(())
    }

    fn process_enum_variants(
        &mut self,
        enum_type: &EnumType,
        shape: &Shape,
    ) -> Result<BTreeMap<u32, Named<VariantFormat>>, Error> {
        let mut variants = BTreeMap::new();
        let mut variant_index = 0u32;

        for variant in enum_type.variants {
            let skip = variant
                .attributes
                .iter()
                .any(|attr| attr.key == "skip" && attr.is_builtin());
            if skip {
                continue;
            }

            let variant_format = self.process_single_variant(variant, shape)?;

            variants.insert(
                variant_index,
                Named {
                    name: variant_display_name(variant),
                    doc: variant.into(),
                    value: variant_format,
                },
            );
            variant_index += 1;
        }

        Ok(variants)
    }

    fn process_single_variant(
        &mut self,
        variant: &Variant,
        shape: &Shape,
    ) -> Result<VariantFormat, Error> {
        if variant.data.fields.is_empty() {
            // Unit variant
            Ok(VariantFormat::Unit)
        } else if variant.data.fields.len() == 1 {
            let is_struct_variant = !variant.data.fields[0]
                .name
                .chars()
                .all(|c| c.is_ascii_digit());

            if is_struct_variant {
                self.process_struct_variant(variant)
            } else {
                self.process_newtype_variant(variant)
            }
        } else {
            self.process_multi_field_variant(variant, shape)
        }
    }

    fn process_newtype_variant(&mut self, variant: &Variant) -> Result<VariantFormat, Error> {
        let field = variant.data.fields[0];
        if let Some(value) = bytes_attribute_format(&field) {
            return Ok(VariantFormat::NewType(Box::new(value)));
        }
        if field.shape().type_identifier == "()" {
            return Ok(VariantFormat::NewType(Box::new(Format::Unit)));
        }

        // A payload that is skipped, as a struct field would be, makes this a unit variant.
        Ok(match self.named_field_format(&field)? {
            None | Some(Format::Unit) => VariantFormat::Unit,
            Some(format) => VariantFormat::NewType(Box::new(format)),
        })
    }

    fn process_multi_field_variant(
        &mut self,
        variant: &Variant,
        shape: &Shape,
    ) -> Result<VariantFormat, Error> {
        // Check if it's a struct variant (named fields) or tuple variant
        let first_field = variant.data.fields[0];
        let is_struct_variant = !first_field.name.chars().all(|c| c.is_ascii_digit());

        if is_struct_variant {
            self.process_struct_variant(variant)
        } else {
            self.process_tuple_variant(variant, shape)
        }
    }

    fn process_struct_variant(&mut self, variant: &Variant) -> Result<VariantFormat, Error> {
        let mut fields = vec![];
        for field in variant.data.fields {
            if field.flags.contains(FieldFlags::SKIP) {
                continue;
            }
            let format = match bytes_attribute_format(field) {
                Some(value) => Some(value),
                None => self.named_field_format(field)?,
            };
            if let Some(value) = format {
                fields.push(Named {
                    name: field_display_name(field),
                    doc: field.into(),
                    value,
                });
            }
        }

        // If all fields were skipped, this is a unit variant
        Ok(if fields.is_empty() {
            VariantFormat::Unit
        } else {
            VariantFormat::Struct(fields)
        })
    }

    fn process_tuple_variant(
        &mut self,
        variant: &Variant,
        shape: &Shape,
    ) -> Result<VariantFormat, Error> {
        // Use the namespace context of the current enum for its variant fields
        let enum_namespace = extract_namespace_from_shape(shape)?;

        let mut formats = vec![];
        for field in variant.data.fields {
            if field.flags.contains(FieldFlags::SKIP) {
                continue;
            }
            if let Some(value) = bytes_attribute_format(field) {
                formats.push(value);
                continue;
            }
            self.push_namespace(enum_namespace.clone());
            let format = self.field_format(field.shape());
            self.pop_namespace();
            if let Some(format) = format? {
                formats.push(format);
            }
        }

        Ok(VariantFormat::Tuple(formats))
    }

    /// Push a namespace action onto the stack (always pushes something)
    fn push_namespace(&mut self, action: NamespaceAction) {
        let context = match action {
            NamespaceAction::SetContext(ctx) => ctx,
            NamespaceAction::Inherit => {
                // Push the current context again (or cleared context if none exists)
                self.namespace_context_stack
                    .last()
                    .cloned()
                    .unwrap_or_else(NamespaceContext::cleared)
            }
        };
        self.namespace_context_stack.push(context);
    }

    /// Pop the current namespace context from the stack
    fn pop_namespace(&mut self) {
        self.namespace_context_stack.pop();
    }

    /// Get the current namespace context (top of stack)
    fn current_namespace_context(&self) -> Option<&NamespaceContext> {
        match self.namespace_context_stack.last() {
            Some(ctx) if ctx.is_cleared() => {
                // This is a "cleared context" marker, so return None
                None
            }
            Some(ctx) => Some(ctx),
            None => None,
        }
    }

    /// The format of a reference to `shape` from the container being built.
    ///
    /// Every container that the reference reaches, however deeply it is nested, is named by
    /// [`Self::get_name_with_mappings`] under the namespace context in force, or under a
    /// transparent wrapper's own namespace (see [`reference_format`]). That is how the container is
    /// named when it is registered, by [`Self::process_nested_types`] under the same context, so the
    /// reference names what is registered: an explicit namespace or ROOT pin is the container's
    /// own, and an unannotated container takes the context's namespace.
    fn reference_to(&mut self, shape: &Shape) -> Result<Format, Error> {
        reference_format(shape, None, &mut |shape, wrapper| {
            let Some(context) = wrapper else {
                return self.get_name_with_mappings(shape);
            };
            self.push_namespace(NamespaceAction::SetContext(context.clone()));
            let name = self.get_name_with_mappings(shape);
            self.pop_namespace();
            name
        })
    }

    /// The format of a field of type `field_shape`, or `None` if the field is to be skipped
    /// because its type cannot be reflected: an opaque type, or an unsupported type (such as
    /// `Result`) anywhere within it.
    fn get_user_type_format(&mut self, field_shape: &Shape) -> Result<Option<Format>, Error> {
        let shape = generated_shape(field_shape);
        // `#[facet(opaque)]` fields use `Def::Undefined`, as user types and pointers also do.
        let opaque = matches!(shape.def, Def::Undefined)
            && !matches!(
                shape.ty,
                Type::User(UserType::Struct(_) | UserType::Enum(_))
                    | Type::Pointer(PointerType::Reference(_) | PointerType::Raw(_))
            );
        if opaque {
            return Ok(None);
        }
        match self.reference_to(field_shape) {
            Ok(format) => Ok(Some(format)),
            // `reference_format` reports an unsupported type, and nothing else, as a
            // `ReflectionError`; naming a container fails with other errors, which propagate.
            Err(Error::ReflectionError { .. }) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Registers every container that a reference to `shape` reaches, under the namespace
    /// context in force: the one that [`Self::reference_to`] named them under.
    ///
    /// The container being built is set aside while they are registered, so that registering
    /// them cannot change it; its reference to `shape` is made by [`Self::reference_to`].
    fn process_nested_types(&mut self, shape: &Shape) -> Result<(), Error> {
        let current = std::mem::take(&mut self.current);
        let result = self.register_reachable(shape);
        self.current = current;
        result
    }

    /// Registers every container that `shape` reaches.
    ///
    /// A transparent wrapper is registered as the type it wraps, under the wrapper's own namespace
    /// if it has one, as [`reference_format`] names it.
    fn register_reachable(&mut self, shape: &Shape) -> Result<(), Error> {
        let (shape, wrapper) = strip_transparent(shape)?;
        let Some(context) = wrapper else {
            return self.register_structure(shape);
        };
        self.push_namespace(NamespaceAction::SetContext(context));
        let result = self.register_structure(shape);
        self.pop_namespace();
        result
    }

    /// [`Self::register_reachable`] for a shape that is not a transparent wrapper.
    fn register_structure(&mut self, shape: &Shape) -> Result<(), Error> {
        if let Def::Option(option_def) = shape.def {
            return self.register_reachable(option_def.t());
        }
        match &shape.ty {
            // Anonymous tuples all share the name `(…)`, so `(i32, u8)` and `(String, bool)` would
            // look like two types claiming one name. No container is registered for a tuple — a
            // reference to it is a `Format::Tuple` — so only its elements are registered.
            Type::User(UserType::Struct(struct_type)) if struct_type.kind == StructKind::Tuple => {
                for field in struct_type.fields {
                    self.register_reachable(field.shape())?;
                }
                return Ok(());
            }
            Type::User(UserType::Struct(_) | UserType::Enum(_)) => {
                return self.format_container(shape);
            }
            Type::Pointer(PointerType::Reference(pt) | PointerType::Raw(pt)) => {
                return self.register_reachable(pt.target);
            }
            _ => {}
        }
        match shape.def {
            Def::List(list_def) => self.register_reachable(list_def.t()),
            Def::Slice(slice_def) => self.register_reachable(slice_def.t()),
            Def::Set(set_def) => self.register_reachable(set_def.t()),
            Def::Array(array_def) => self.register_reachable(array_def.t()),
            Def::Map(map_def) => {
                self.register_reachable(map_def.k())?;
                self.register_reachable(map_def.v())
            }
            Def::Pointer(PointerDef {
                pointee: Some(pointee),
                ..
            }) => self.register_reachable(pointee),
            _ => Ok(()),
        }
    }

    fn get_name_with_mappings(&mut self, shape: &Shape) -> Result<QualifiedTypeName, Error> {
        // First check if there's a mapping for this type
        let base_key = QualifiedTypeName::root(shape.type_identifier.to_string());
        if let Some(mapping) = self.name_mappings.get(&base_key)
            && mapping.id == generated_shape(shape).id
        {
            return Ok(mapping.renamed.clone());
        }

        // Get the original name (which includes explicit namespace annotations)
        let original_name = get_name(shape)?;

        // Check if the type has an explicit namespace annotation (type-level explicit)
        let has_explicit_namespace = extract_namespace_from_shape(shape)?.is_explicit();

        if has_explicit_namespace {
            // Type-level explicit namespace annotation overrides everything (including field-level explicit)
            self.check_namespace_ambiguity(&original_name, true)?;
            return Ok(original_name);
        }

        // If no type-level explicit namespace annotation, apply current context if available
        if let Some(namespace_context) = self.current_namespace_context() {
            match &namespace_context.namespace {
                Namespace::Root => {
                    // Context is explicitly root, use root namespace
                    let root_name = QualifiedTypeName::root(original_name.name.clone());
                    self.check_namespace_ambiguity(&root_name, namespace_context.is_explicit())?;
                    return Ok(root_name);
                }
                Namespace::Named(context_ns) => {
                    // Apply the context namespace
                    let namespaced_name = QualifiedTypeName::namespaced(
                        context_ns.clone(),
                        original_name.name.clone(),
                    );

                    self.check_namespace_ambiguity(
                        &namespaced_name,
                        namespace_context.is_explicit(),
                    )?;

                    return Ok(namespaced_name);
                }
            }
        }

        // Fall back to the original name (which will be root if no annotation)
        Ok(original_name)
    }

    fn check_namespace_ambiguity(
        &mut self,
        new_name: &QualifiedTypeName,
        new_has_explicit_namespace: bool,
    ) -> Result<(), Error> {
        for (existing_name, existing_has_explicit_namespace) in &self.type_namespace_sources {
            if existing_name.name == new_name.name
                && existing_name.namespace != new_name.namespace
                && (!matches!(existing_name.namespace, Namespace::Root)
                    && !matches!(new_name.namespace, Namespace::Root))
            {
                let at_least_one_namespace_inherited =
                    !new_has_explicit_namespace || !*existing_has_explicit_namespace;
                if at_least_one_namespace_inherited {
                    return Err(Error::AmbiguousNamespaceInheritance {
                        type_name: new_name.name.clone(),
                        existing_namespace: existing_name.namespace.to_string(),
                        new_namespace: new_name.namespace.to_string(),
                    });
                }
            }
        }

        self.type_namespace_sources
            .insert(new_name.clone(), new_has_explicit_namespace);

        Ok(())
    }
}

/// Checks that every [`Format::TypeName`] in the registry names a registered container.
///
/// A reference that names nothing would reach the generators as a type that no module declares.
///
/// # Errors
/// Returns [`Error::DanglingTypeReference`] for the first reference that names no container.
fn check_references(registry: &Registry) -> Result<(), Error> {
    for (name, container) in registry {
        let check =
            |location: String, format: &Format| check_reference(registry, name, &location, format);
        match container {
            ContainerFormat::UnitStruct(_) => {}
            ContainerFormat::NewTypeStruct(format, _) => check("0".to_string(), format)?,
            ContainerFormat::TupleStruct(formats, _) => {
                for (index, format) in formats.iter().enumerate() {
                    check(index.to_string(), format)?;
                }
            }
            ContainerFormat::Struct(fields, _) => {
                for field in fields {
                    check(field.name.clone(), &field.value)?;
                }
            }
            ContainerFormat::Enum(variants, _, _) => {
                for variant in variants.values() {
                    let variant_name = &variant.name;
                    match &variant.value {
                        VariantFormat::Variable(_) | VariantFormat::Unit => {}
                        VariantFormat::NewType(format) => check(variant_name.clone(), format)?,
                        VariantFormat::Tuple(formats) => {
                            for (index, format) in formats.iter().enumerate() {
                                check(format!("{variant_name}.{index}"), format)?;
                            }
                        }
                        VariantFormat::Struct(fields) => {
                            for field in fields {
                                check(format!("{variant_name}.{}", field.name), &field.value)?;
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Checks every type name in `format`, which is found at `location` in the container `name`.
fn check_reference(
    registry: &Registry,
    name: &QualifiedTypeName,
    location: &str,
    format: &Format,
) -> Result<(), Error> {
    format.visit(&mut |format| {
        if let Format::TypeName(missing) = format
            && !registry.contains_key(missing)
        {
            return Err(Error::DanglingTypeReference {
                name: name.name.clone(),
                namespace: name.namespace.to_string(),
                location: location.to_string(),
                missing_name: missing.name.clone(),
                missing_namespace: missing.namespace.to_string(),
            });
        }
        Ok(())
    })
}

fn get_name(shape: &Shape) -> Result<QualifiedTypeName, Error> {
    // Check type_tag first (is this where facet rename is stored?)
    if let Some(type_tag) = shape.type_tag {
        return Ok(QualifiedTypeName::root(type_tag.to_string()));
    }

    let shape_namespace = extract_namespace_from_shape(shape)?;

    // Check for transparent via repr
    if is_transparent_shape(shape)
        && let Some(inner) = shape.inner
    {
        return get_name(inner);
    }

    // Determine the base name — use the built-in `rename` field if present,
    // then check attributes for a rename (facet derive stores it there),
    // otherwise fall back to the type identifier.
    let base_name = shape.rename.map_or_else(
        || {
            extract_rename_from_shape_attributes(shape)
                .map_or_else(|| shape.type_identifier.to_string(), ToString::to_string)
        },
        ToString::to_string,
    );

    // Apply namespace - only use explicit namespace annotations, no inheritance
    Ok(match shape_namespace {
        NamespaceAction::SetContext(ctx) => match &ctx.namespace {
            Namespace::Root => QualifiedTypeName::root(base_name),
            Namespace::Named(name) => QualifiedTypeName::namespaced(name.clone(), base_name),
        },
        NamespaceAction::Inherit => QualifiedTypeName::root(base_name),
    })
}

fn type_to_format(shape: &Shape) -> Result<Option<Format>, Error> {
    let format = match &shape.ty {
        Type::Primitive(primitive) => Some(match primitive {
            PrimitiveType::Boolean => Format::Bool,
            PrimitiveType::Numeric(numeric_type) => match numeric_type {
                NumericType::Float => {
                    // Determine float type based on size or type identifier
                    match shape.type_identifier {
                        "f32" => Format::F32,
                        "f64" => Format::F64,
                        _ => unimplemented!("Unsupported float type: {}", shape.type_identifier),
                    }
                }
                NumericType::Integer { signed } => {
                    // Get the size from the layout
                    let size_bytes = shape
                        .layout
                        .sized_layout()
                        .map_err(|_| Error::LayoutUnsized(shape.to_string()))?
                        .size();
                    let size_bits = size_bytes * 8;

                    match (*signed, size_bits) {
                        (false, 8) => Format::U8,
                        (false, 16) => Format::U16,
                        (false, 32) => Format::U32,
                        (false, 64) => Format::U64,
                        (false, 128) => Format::U128,
                        (true, 8) => Format::I8,
                        (true, 16) => Format::I16,
                        (true, 32) => Format::I32,
                        (true, 64) => Format::I64,
                        (true, 128) => Format::I128,
                        _ => unimplemented!(
                            "Unsupported integer type: {size_bits} bits, signed: {signed}"
                        ),
                    }
                }
            },
            PrimitiveType::Textual(textual_type) => match textual_type {
                TextualType::Str => Format::Str,
                TextualType::Char => Format::Char,
            },
            PrimitiveType::Never => {
                unimplemented!("Never type not supported: {}", shape.type_identifier)
            }
        }),
        Type::User(UserType::Opaque) => match shape.type_identifier {
            "String" | "DateTime<Utc>" => Some(Format::Str),
            "Uuid" => Some(Format::Uuid),
            _ => None,
        },
        // Handle () unit type which in facet 0.44.1 appears as User(Struct(Tuple, []))
        Type::User(UserType::Struct(st))
            if st.kind == StructKind::Tuple && st.fields.is_empty() =>
        {
            Some(Format::Unit)
        }
        _ => unimplemented!(
            "Unsupported type for scalar format: {}: {:?}",
            shape.type_identifier,
            shape.ty
        ),
    };

    Ok(format)
}

/// Extract a rename value from shape attributes.
///
/// In the latest facet, `#[facet(rename = "...")]` on a container is stored
/// in `shape.attributes` rather than `shape.rename`. This helper checks the
/// attributes for a builtin `rename` key and returns the value if found.
fn extract_rename_from_shape_attributes(shape: &Shape) -> Option<&'static str> {
    for attr in shape.attributes {
        if attr.ns.is_none()
            && attr.key == "rename"
            && let Some(s) = attr.get_as::<&str>()
        {
            return Some(s);
        }
    }
    None
}

fn extract_enum_tagging(shape: &Shape) -> EnumTagging {
    match (shape.tag, shape.content) {
        (Some(tag), Some(content)) => EnumTagging::Adjacent {
            tag: tag.to_string(),
            content: content.to_string(),
        },
        (Some(tag), None) => EnumTagging::Internal {
            tag: tag.to_string(),
        },
        _ => EnumTagging::External,
    }
}

fn extract_namespace_from_shape(shape: &Shape) -> Result<NamespaceAction, Error> {
    for attr in shape.attributes {
        if let Some(action) = extract_namespace_from_attr(attr)? {
            return Ok(action);
        }
    }
    Ok(NamespaceAction::Inherit)
}

fn extract_namespace_from_field_attributes(field: &Field) -> Result<NamespaceAction, Error> {
    for attr in field.attributes {
        if let Some(action) = extract_namespace_from_attr(attr)? {
            return Ok(action);
        }
    }
    Ok(NamespaceAction::Inherit)
}

/// Returns the display name for a field, respecting `#[facet(rename = "...")]`.
///
/// If the field has a `rename` value, that is used; otherwise falls back to `field.name`.
fn field_display_name(field: &Field) -> String {
    field.rename.unwrap_or(field.name).to_string()
}

/// Returns the display name for a variant, respecting `#[facet(rename = "...")]`.
///
/// If the variant has a `rename` value, that is used; otherwise falls back to `variant.name`.
fn variant_display_name(variant: &facet::Variant) -> String {
    variant.rename.unwrap_or(variant.name).to_string()
}

/// Extract a namespace action from a single `fg::namespace` extension attribute.
///
/// - `#[facet(fg::namespace = "MyNs")]` → `SetContext(explicit(Named("MyNs")))`
/// - `#[facet(fg::namespace)]` (no value) → `SetContext(cleared())`
/// - Any other attribute → `None` (not a namespace attr)
fn extract_namespace_from_attr(attr: &facet::Attr) -> Result<Option<NamespaceAction>, Error> {
    if attr.ns != Some("fg") || attr.key != "namespace" {
        return Ok(None);
    }

    // The data is stored as fg::Attr::Namespace(Option<&'static str>)
    let Some(gen_attr) = attr.get_as::<fg::Attr>() else {
        return Err(Error::InvalidNamespaceFormat);
    };

    match gen_attr {
        fg::Attr::Namespace(Some(ns_str)) => {
            static VALID_IDENT: LazyLock<Regex> =
                LazyLock::new(|| Regex::new(r"^[a-zA-Z_]\w*$").expect("Invalid regex"));
            if !VALID_IDENT.is_match(ns_str) {
                return Err(Error::InvalidNamespaceIdentifier);
            }
            Ok(Some(NamespaceAction::SetContext(
                NamespaceContext::explicit(Namespace::Named(ns_str.to_string())),
            )))
        }
        fg::Attr::Namespace(None) => Ok(Some(NamespaceAction::SetContext(
            NamespaceContext::cleared(),
        ))),
        _ => Ok(None),
    }
}

fn is_transparent_shape(shape: &Shape) -> bool {
    shape
        .attributes
        .iter()
        .any(|attr| attr.ns.is_none() && attr.key == "transparent")
}

fn bytes_attribute_format(field: &Field) -> Option<Format> {
    let mut shape = field.shape();
    let is_bytes_attr = |field: &Field| {
        field
            .attributes
            .iter()
            .any(|attr| attr.key == "bytes" && attr.ns == Some("fg"))
    };
    let mut is_transparent_bytes = || {
        if is_transparent_shape(shape) {
            match shape.ty {
                Type::User(ty) => match ty {
                    UserType::Struct(ty) => match ty.kind {
                        StructKind::TupleStruct if ty.fields.len() == 1 => {
                            let field = ty.fields[0];
                            shape = field.shape();
                            is_bytes_attr(&field)
                        }
                        _ => false,
                    },
                    UserType::Enum(_) => todo!(),
                    UserType::Union(_) => todo!(),
                    UserType::Opaque => false,
                },
                _ => false,
            }
        } else {
            false
        }
    };
    if !is_bytes_attr(field) && !is_transparent_bytes() {
        return None;
    }

    let (is_option, field_shape) = {
        let base_shape = shape;
        if let Def::Option(opt) = base_shape.def {
            (true, opt.t)
        } else {
            (false, base_shape)
        }
    };

    let format = || {
        if is_option {
            Format::Option(Box::new(Format::Bytes))
        } else {
            Format::Bytes
        }
    };

    // Handle bytes attribute for Vec<u8>
    if field_shape.type_identifier == "Vec" {
        // Check if it's actually Vec<u8> by examining the definition
        if let Def::List(list_def) = field_shape.def {
            let inner_shape = list_def.t();
            if inner_shape.type_identifier == "u8" {
                return Some(format());
            }
        }
    }

    if field_shape.type_identifier == "Bytes" {
        return Some(format());
    }

    // Handle fixed byte arrays
    if let Def::Array(ArrayDef { t, .. }) = field_shape.def
        && t.type_identifier == "u8"
    {
        return Some(format());
    }

    // Handle bytes attribute for &[u8] slices
    if let Type::Pointer(PointerType::Reference(pd) | PointerType::Raw(pd)) = &field_shape.ty {
        let target_shape = pd.target;
        if let Def::Slice(slice_def) = target_shape.def {
            let element_shape = slice_def.t();
            if element_shape.type_identifier == "u8" {
                return Some(format());
            }
        }
    }
    eprintln!(
        "{} is not a valid bytes attribute",
        field_shape.type_identifier
    );
    None
}

/// Names a container, given the namespace context of a transparent wrapper around it, if any.
type NameOf<'a> =
    dyn FnMut(&Shape, Option<&NamespaceContext>) -> Result<QualifiedTypeName, Error> + 'a;

/// The [`Format`] of a reference to `shape`, naming every container it reaches with `name_of`.
///
/// This is the structure of the reference alone: `Option`, sequences, sets, maps, tuples, arrays,
/// pointers and transparent wrappers are unwrapped, and a container is named by `name_of`, so that
/// however deeply a container is nested its name comes from one place.
///
/// A transparent wrapper is referred to as the type it wraps, however long the chain of wrappers.
/// The innermost wrapper with an `fg::namespace` of its own sets the namespace context for that
/// type and everything within it, just as a container's own namespace does for its fields, so
/// `name_of` is given that context (`wrapper`) along with the container to name.
fn reference_format(
    shape: &Shape,
    wrapper: Option<&NamespaceContext>,
    name_of: &mut NameOf,
) -> Result<Format, Error> {
    let (shape, inner_wrapper) = strip_transparent(shape)?;
    let wrapper = inner_wrapper.as_ref().or(wrapper);

    // A struct or enum is a container whatever its `Def`, as `format_container` registers it:
    // facet gives some std types that are user structs, such as `Range` and `PhantomData`,
    // `Def::Scalar`. Only `Option` (an enum with `Def::Option`) and anonymous tuples are
    // structural.
    let is_container = match &shape.ty {
        Type::User(UserType::Struct(struct_type)) => struct_type.kind != StructKind::Tuple,
        Type::User(UserType::Enum(_)) => !matches!(shape.def, Def::Option(_)),
        _ => false,
    };
    if is_container {
        return Ok(Format::TypeName(name_of(shape, wrapper)?));
    }

    let format = match shape.def {
        Def::Scalar => match type_to_format(shape)? {
            Some(format) => format,
            None => {
                return Err(Error::ReflectionError {
                    type_name: shape.type_identifier.to_string(),
                    message: "Scalar type is not supported and should be skipped".to_string(),
                });
            }
        },
        Def::List(inner_list_def) => Format::Seq(Box::new(reference_format(
            inner_list_def.t(),
            wrapper,
            name_of,
        )?)),
        Def::Option(option_def) => Format::Option(Box::new(reference_format(
            option_def.t(),
            wrapper,
            name_of,
        )?)),
        Def::Map(map_def) => Format::Map {
            key: Box::new(reference_format(map_def.k(), wrapper, name_of)?),
            value: Box::new(reference_format(map_def.v(), wrapper, name_of)?),
        },
        Def::Set(set_def) => {
            Format::Set(Box::new(reference_format(set_def.t(), wrapper, name_of)?))
        }
        Def::Array(array_def) => Format::TupleArray {
            content: Box::new(reference_format(array_def.t(), wrapper, name_of)?),
            size: array_def.n,
        },
        Def::Undefined => {
            if let Type::User(UserType::Struct(struct_type)) = &shape.ty
                && struct_type.kind == StructKind::Tuple
                && !struct_type.fields.is_empty()
            {
                let mut tuple_formats = vec![];
                for field in struct_type.fields {
                    tuple_formats.push(reference_format(field.shape(), wrapper, name_of)?);
                }
                return Ok(Format::Tuple(tuple_formats));
            }

            if shape.type_identifier == "()" {
                Format::Unit
            } else if let Type::Pointer(PointerType::Reference(pt) | PointerType::Raw(pt)) =
                &shape.ty
            {
                // For pointer types like &'static str, get the format of the target type
                reference_format(pt.target, wrapper, name_of)?
            } else {
                Format::TypeName(name_of(shape, wrapper)?)
            }
        }
        Def::Slice(slice_def) => {
            Format::Seq(Box::new(reference_format(slice_def.t(), wrapper, name_of)?))
        }
        Def::Pointer(pointer_def) => {
            // Handle Pointer (Box, Arc, etc.) by recursively processing the inner type
            if let Some(inner_shape) = pointer_def.pointee {
                reference_format(inner_shape, wrapper, name_of)?
            } else {
                // Fallback for pointers without a known pointee
                Format::Unit
            }
        }
        _ => {
            // For example `Result`, which is not supported
            return Err(Error::ReflectionError {
                type_name: shape.type_identifier.to_string(),
                message: "Type is not supported and should be skipped".to_string(),
            });
        }
    };

    Ok(format)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "namespace_tests.rs"]
mod namespace_tests;

#[cfg(test)]
#[path = "reference_tests.rs"]
mod reference_tests;
