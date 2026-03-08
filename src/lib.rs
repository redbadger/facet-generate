pub mod error;
pub mod generation;
pub mod reflection;

#[cfg(all(test, feature = "generate"))]
mod tests;

use std::collections::BTreeMap;

use crate::{
    error::Error,
    reflection::format::{ContainerFormat, QualifiedTypeName},
};

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// The registry of reflected types — a flat map from qualified type names to their container formats.
///
/// Built by [`reflection::RegistryBuilder`] (typically via the [`reflect!`] macro) and consumed by
/// language-specific code generators in [`generation`].
///
/// Only named container types (structs and enums) get top-level entries. Primitives, `Option`,
/// `Vec`, `Map`, etc. are represented inline as [`Format`](reflection::format::Format) variants
/// within the containers that use them. Cross-type references are expressed as
/// `Format::TypeName(QualifiedTypeName)` — symbolic lookups back into this same map.
///
/// Keys are namespace-qualified, so a type `Foo` in the root namespace and a type `Foo` in
/// namespace `Bar` are separate entries. For example, in Kotlin these would generate as `Foo`
/// and `Bar.Foo` respectively.
pub type Registry = BTreeMap<QualifiedTypeName, ContainerFormat>;

// TODO: Consider removing `#[macro_export]` — this macro is only used in
// tests but currently leaks into the public API.
/// Test/convenience macro: reflects the given types and emits code for each
/// container using the specified language tag and encoding.
///
/// Returns `anyhow::Result<String>` containing the generated source.
///
/// ```ignore
/// let code = emit!(MyStruct, MyEnum as Kotlin with Encoding::Json)?;
/// ```
///
/// This skips the [`Module`](generation::module::Module) header (no `package`
/// or `import` statements) — it only emits the type declarations. Useful in
/// tests to assert on individual type output without the file-level boilerplate.
#[macro_export]
macro_rules! emit {
    ($($ty:ident),* as $language:ident with $encoding:path) => {
        || -> anyhow::Result<String> {
            use $crate::generation::{Container, indent::{IndentConfig, IndentedWriter}};
            use std::io::Write as _;
            let mut out = Vec::new();
            let mut w = IndentedWriter::new(&mut out, IndentConfig::Space(4));
            let registry = $crate::reflect!($($ty),*)?;
            for container in registry.iter().map(Container::from) {
                writeln!(&mut w)?;
                let lang = $language::new($encoding);
                container.write(&mut w, lang)?;
            }
            let out = String::from_utf8(out)?;

            Ok(out)
        }()
    };
}

/// **Deprecated since 0.16.0:** The Java generator is deprecated. Use the Kotlin generator instead.
#[macro_export]
#[deprecated(
    since = "0.16.0",
    note = "The Java generator is deprecated. Use the Kotlin generator instead."
)]
macro_rules! emit_java {
    ($($ty:ident),* as $encoding:path) => {
        #[allow(deprecated)]
        || -> anyhow::Result<String> {
            use $crate::generation::{Encoding, indent::{IndentConfig, IndentedWriter}};
            let mut out = Vec::new();
            let w = IndentedWriter::new(&mut out, IndentConfig::Space(4));
            let config = $crate::generation::CodeGeneratorConfig::new("com.example".to_string())
                .with_encoding($encoding);
            let generator = $crate::generation::java::CodeGenerator::new(&config);
            let mut emitter = $crate::generation::java::emitter::JavaEmitter {
                out: w,
                generator: &generator,
                current_namespace: Vec::new(),
                current_reserved_names: HashMap::new(),
            };
            let registry = $crate::reflect!($($ty),*)?;
            for (name, format) in &registry {
                emitter.output_container(&name.name, format).unwrap();
            }
            let out = String::from_utf8(out)?;

            Ok(out)
        }()
    };
}

/// Reflects one or more types into a [`Registry`], recursively capturing all reachable types.
///
/// This is a convenience wrapper around [`RegistryBuilder`](reflection::RegistryBuilder) —
/// used directly by the [`emit!`] macro and available for cases where you need the registry
/// without code generation.
///
/// ```ignore
/// let registry = reflect!(MyStruct, MyEnum)?;
/// ```
#[macro_export]
macro_rules! reflect {
    ($($ty:ident),*) => {
        || -> anyhow::Result<std::collections::BTreeMap<$crate::reflection::format::QualifiedTypeName, $crate::reflection::format::ContainerFormat>> {
            let registry = $crate::reflection::RegistryBuilder::new()
                $(.add_type::<$ty>().map_err(|e| anyhow::anyhow!("failed to add type {}: {}", stringify!($ty), e))?)*
                .build()
                .map_err(|e| anyhow::anyhow!("failed to build registry: {e}"))?;
            Ok(registry)
        }()
    };
}

// TODO: Consider removing `#[macro_export]` — this macro is only used in
// tests but currently leaks into the public API.
/// Test-only macro for multi-namespace generation tests.
///
/// Reflects `$facet`, splits the resulting registry by namespace (expecting
/// exactly two namespaces), and runs the full [`CodeGen`](generation::CodeGen)
/// pipeline for each. Returns `(String, String)` — the generated source for
/// the non-root module and the root module, sorted alphabetically by module
/// name.
///
/// This exercises the complete generator path *including* the
/// [`Module`](generation::module::Module) header (package declaration,
/// imports), unlike [`emit!`] which skips it.
#[cfg(test)]
#[macro_export]
macro_rules! emit_two_modules {
    ($generator:ty, $facet:ident, $root:expr) => {{
        use $crate::generation::CodeGen;
        use $crate::generation::module::{self, Module};
        use $crate::{Registry, reflect};

        fn emit_module<'a, G: CodeGen<'a>>(module: &'a Module, registry: &Registry) -> String {
            let mut out = Vec::new();
            let mut generator = G::new(module.config());
            generator.write_output(&mut out, registry).unwrap();
            String::from_utf8(out).unwrap()
        }

        let registry = reflect!($facet).unwrap();
        let mut modules: Vec<_> = module::split($root, &registry).into_iter().collect();
        modules.sort_by(|a, b| a.0.config().module_name.cmp(&b.0.config().module_name));

        let modules: [(Module, Registry); 2] = modules.try_into().expect("Two modules expected");
        let [(other_module, other_registry), (root_module, root_registry)] = modules;

        let module_1 = emit_module::<$generator>(&other_module, &other_registry);
        let module_2 = emit_module::<$generator>(&root_module, &root_registry);
        (module_1, module_2)
    }};
}
