//! Top-level orchestrator for Kotlin code generation.
//!
//! [`KotlinCodeGenerator`] implements [`CodeGenerator`] and is the entry point for
//! producing a single Kotlin source file from a [`Registry`].

use crate::{
    Registry,
    generation::{
        CodeGenerator, CodeGeneratorConfig, Container, Emitter, config::PackageLocation,
        indent::IndentedWriter, kotlin::emitter::Kotlin, module::Module,
    },
    reflection::format::{Format, FormatHolder, Namespace, QualifiedTypeName},
};
use std::io::{Result, Write};

/// Kotlin code generator — holds a reference to the shared
/// [`CodeGeneratorConfig`] and implements [`CodeGenerator`].
pub struct KotlinCodeGenerator<'a> {
    /// Language-independent configuration (encoding, module name, external
    /// packages, etc.).
    pub(crate) config: &'a CodeGeneratorConfig,
}

impl<'a> CodeGenerator<'a> for KotlinCodeGenerator<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        KotlinCodeGenerator::new(config)
    }

    fn write_output<W: std::io::Write>(
        &mut self,
        writer: &mut W,
        registry: &Registry,
    ) -> Result<()> {
        self.output(writer, registry)
    }
}

impl<'a> KotlinCodeGenerator<'a> {
    /// Create a Kotlin code generator for the given config
    #[must_use]
    pub const fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self { config }
    }

    /// Produce a complete Kotlin source file for the given `registry`.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying writer fails.
    pub fn output(&self, out: &mut impl Write, registry: &Registry) -> Result<()> {
        let w = &mut IndentedWriter::new(out, self.config.indent);

        let mut config = self.config.clone();
        config.update_from(registry);

        let lang = Kotlin::new(&config, registry);

        Module::new(&config).write(w, &lang)?;

        for (i, container) in Self::update_qualified_names(&config, registry)
            .iter()
            .map(Container::from)
            .enumerate()
        {
            if i > 0 {
                writeln!(w)?;
            }
            container.write(w, &lang)?;
        }
        Ok(())
    }

    /// Rewrites every [`QualifiedTypeName`] in the registry to a fully-qualified
    /// Kotlin package path, returning a new registry.
    ///
    /// The registry coming in from reflection uses short, language-agnostic
    /// names like `Namespace::Named("Other") + "MyType"`. Kotlin needs these
    /// turned into dot-separated package paths so the emitter can write e.g.
    /// `com.example.other.Other.MyType` in type references and deserialize
    /// calls.
    ///
    /// # Resolution rules (checked in this order)
    ///
    /// 1. **External package with a [`PackageLocation::Path`]** — the
    ///    configured path replaces the namespace prefix.
    ///    `Other::MyType` with path `com.acme.other` → `com.acme.other.Other.MyType`
    ///
    /// 2. **External definition in a different namespace** — prefixed with
    ///    the current module name.
    ///    Module `com.example.main`, namespace `auth` → `com.example.main.auth.User`
    ///
    /// 3. **Same namespace as the current module** — collapsed to just the
    ///    module name (no double-nesting).
    ///    Module `com.example.other`, namespace `other` → `com.example.other.LocalType`
    ///
    /// 4. **[`Namespace::Root`]** — uses the current module name.
    ///    Module `com.example.service` → `com.example.service.RootType`
    fn update_qualified_names(config: &CodeGeneratorConfig, registry: &Registry) -> Registry {
        let mut updated_registry = registry.clone();

        for container_format in updated_registry.values_mut() {
            let _ = container_format.visit_mut(&mut |format| {
                if let Format::TypeName(qualified_name) = format {
                    match &qualified_name.namespace {
                        Namespace::Named(namespace) => {
                            let namespace = namespace.clone();
                            // First check if this namespace has an external package configuration with a Path
                            let external_package_handled = if let Some(external_package) =
                                config.external_packages.get(&namespace)
                            {
                                if let PackageLocation::Path(path) = &external_package.location {
                                    let full_namespace = format!("{path}.{namespace}");
                                    *qualified_name = QualifiedTypeName::namespaced(
                                        full_namespace,
                                        qualified_name.name.clone(),
                                    );
                                    true
                                } else {
                                    // PackageLocation::Url is ignored for Kotlin generation - fall through
                                    false
                                }
                            } else {
                                false
                            };

                            if !external_package_handled {
                                // Check if this type's namespace matches the current module's namespace
                                let current_leaf_namespace = config
                                    .module_name()
                                    .rsplit_once('.')
                                    .map_or_else(|| config.module_name(), |(_, leaf)| leaf);

                                if config.external_definitions.contains_key(&namespace)
                                    && namespace != current_leaf_namespace
                                {
                                    // For external types, build full path: current_module.namespace
                                    let full_namespace =
                                        format!("{}.{namespace}", config.module_name());
                                    *qualified_name = QualifiedTypeName::namespaced(
                                        full_namespace,
                                        qualified_name.name.clone(),
                                    );
                                } else if namespace == current_leaf_namespace {
                                    // For same-module types, use current module name only
                                    *qualified_name = QualifiedTypeName::namespaced(
                                        config.module_name().to_string(),
                                        qualified_name.name.clone(),
                                    );
                                } else {
                                    // For other local types with named namespace, preserve the namespace
                                    let full_namespace =
                                        format!("{}.{namespace}", config.module_name());
                                    *qualified_name = QualifiedTypeName::namespaced(
                                        full_namespace,
                                        qualified_name.name.clone(),
                                    );
                                }
                            }
                        }
                        Namespace::Root => {
                            // Root namespace types get current module name
                            *qualified_name = QualifiedTypeName::namespaced(
                                config.module_name().to_string(),
                                qualified_name.name.clone(),
                            );
                        }
                    }
                }
                Ok(())
            });
        }

        updated_registry
    }
}

#[cfg(test)]
mod tests;
