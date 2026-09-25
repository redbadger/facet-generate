//! Top-level orchestrator for C# code generation.
//!
//! [`CSharpCodeGenerator`] implements [`CodeGenerator`](super::super::CodeGenerator) to produce a
//! complete C# source file from a [`Registry`](crate::Registry). It resolves
//! qualified type names using the dotted-namespace convention, then delegates
//! AST-to-source rendering to the [emitter](super::emitter) layer.

use std::io::{Result, Write};

use std::sync::Arc;

use crate::{
    Registry,
    generation::{
        CodeGenerator, CodeGeneratorConfig, Container, Emitter,
        csharp::{
            emitter::{CSharp, write_module_header},
            naming,
        },
        indent::IndentedWriter,
        module::Module,
        naming::check_reserved_names,
        plugin::{CompanionFile, EmitterPlugin, render_companion_files},
    },
    reflection::format::{Format, FormatHolder, Namespace, QualifiedTypeName},
};

/// Main configuration object for C# code generation.
///
/// Wraps a [`CodeGeneratorConfig`] and implements [`CodeGenerator`] to provide the
/// entry point for producing C# source from a registry.
pub struct CSharpCodeGenerator<'a> {
    /// Language-independent configuration.
    pub(crate) config: &'a CodeGeneratorConfig,
    /// Pre-built plugins to apply during code generation.
    pub(crate) plugins: Vec<Arc<dyn EmitterPlugin<CSharp>>>,
}

impl<'a> CodeGenerator<'a> for CSharpCodeGenerator<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self {
            config,
            plugins: vec![],
        }
    }

    fn write_output<W: Write>(&mut self, writer: &mut W, registry: &Registry) -> Result<()> {
        self.output(writer, registry)
    }
}

impl<'a> CSharpCodeGenerator<'a> {
    /// Create a C# code generator with no plugins.
    ///
    /// Call [`with_plugins`](Self::with_plugins) to add serialization plugins.
    #[must_use]
    pub fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self {
            config,
            plugins: vec![],
        }
    }

    /// Set pre-built plugins, returning the modified generator.
    #[must_use]
    pub fn with_plugins(mut self, plugins: Vec<Arc<dyn EmitterPlugin<CSharp>>>) -> Self {
        self.plugins = plugins;
        self
    }

    /// Output type definitions for `registry`.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `out` fails.
    pub fn output(&self, out: &mut impl Write, registry: &Registry) -> Result<()> {
        let w = &mut IndentedWriter::new(out, self.config.indent);

        let mut config = self.config.clone();
        config.update_from(registry);
        config.requalify_enums(registry, Self::requalify);
        check_reserved_names(registry, &naming::RULES)?;

        let updated_registry = Self::update_qualified_names(&config, registry);
        let mut lang = CSharp::new(&config, &updated_registry);
        for p in &self.plugins {
            lang = lang.with_plugin(p.clone());
        }

        Module::new(&config).write(w, &lang)?;

        for (index, container) in updated_registry.iter().map(Container::from).enumerate() {
            if index > 0 {
                writeln!(w)?;
            }
            container.write(w, &lang)?;
        }

        Ok(())
    }

    /// Render the companion files contributed by the plugins for `registry`.
    ///
    /// Each returned [`CompanionFile`] carries the file's final contents: the
    /// same module header [`output`](Self::output) writes (minus the module
    /// helpers), merged with the file's own imports, followed by the plugin's
    /// body. The installer writes them into the module's namespace directory.
    ///
    /// # Errors
    ///
    /// Returns an error if rendering a header fails.
    pub fn companion_files(&self, registry: &Registry) -> Result<Vec<CompanionFile>> {
        let mut config = self.config.clone();
        config.update_from(registry);
        config.requalify_enums(registry, Self::requalify);

        let updated_registry = Self::update_qualified_names(&config, registry);
        let mut lang = CSharp::new(&config, &updated_registry);
        for p in &self.plugins {
            lang = lang.with_plugin(p.clone());
        }

        render_companion_files(lang.plugins(), &config, |imports| {
            let mut header = Vec::new();
            write_module_header(
                &mut IndentedWriter::new(&mut header, config.indent),
                &config,
                &lang,
                imports,
            )?;
            String::from_utf8(header).map_err(std::io::Error::other)
        })
    }

    /// Update [`QualifiedTypeName`] instances for C#'s dotted-namespace rules.
    ///
    /// Every namespace is declared under the *root package* (the parent the
    /// installer nested this module under, or the module itself when it has
    /// none), so references are rooted there rather than at `module_name()`,
    /// which for a namespaced module already ends in its own namespace.
    ///
    /// 1. **Same namespace** — a `Named("Users")` reference inside the module
    ///    of namespace `Users`, `Company.Models.Users`, is stripped to `Root`
    ///    (bare name). Which namespace is the module's own comes from its
    ///    config (see [`CodeGeneratorConfig::generates`]), not from the last
    ///    segment of its name: inside the root module `Example.Kv`, a
    ///    `Named("Kv")` reference is another namespace (rule 2).
    /// 2. **Other namespace** — a `Named("Payments")` reference inside module
    ///    `Company.Models`, or inside its child `Company.Models.Users`, becomes
    ///    `Named("Company.Models.Payments")`.
    /// 3. **Root from a dotted root module** — a `Root` reference inside module
    ///    `Company.Models` is promoted to `Named("Company.Models")`, and stays
    ///    bare inside an undotted one (`Example`).
    /// 4. **Root from a namespaced module** — a `Root` reference inside
    ///    `Company.Models.Users` (parent `Company.Models`) becomes
    ///    `Named("Company.Models")`, and inside `Example.Users` (parent
    ///    `Example`) becomes `Named("Example")`, so that it cannot mean a
    ///    same-named type of the module's own.
    fn update_qualified_names(config: &CodeGeneratorConfig, registry: &Registry) -> Registry {
        let mut updated_registry = registry.clone();

        for container_format in updated_registry.values_mut() {
            Self::requalify_type_names(config, container_format);
        }

        updated_registry
    }

    /// Rewrites every type reference in `holder` with
    /// [`requalify`](Self::requalify), exported to plugins for a [`Format`] as
    /// [`csharp::requalify_format`](crate::generation::csharp::requalify_format).
    pub(crate) fn requalify_type_names(
        config: &CodeGeneratorConfig,
        holder: &mut impl FormatHolder,
    ) {
        let _ = holder.visit_mut(&mut |format| {
            if let Format::TypeName(qualified_name) = format {
                *qualified_name = Self::requalify(config, qualified_name);
            }
            Ok(())
        });
    }

    /// The spelling [`update_qualified_names`](Self::update_qualified_names)
    /// gives a reference to `name`, exported to plugins as
    /// [`csharp::requalify`](crate::generation::csharp::requalify).
    pub(crate) fn requalify(
        config: &CodeGeneratorConfig,
        name: &QualifiedTypeName,
    ) -> QualifiedTypeName {
        match &name.namespace {
            Namespace::Named(namespace) => {
                if config.is_own_namespace(namespace) {
                    QualifiedTypeName::root(name.name.clone())
                } else {
                    QualifiedTypeName::namespaced(
                        format!("{}.{}", config.root_package(), namespace),
                        name.name.clone(),
                    )
                }
            }
            Namespace::Root => {
                if config.parent.is_some() || config.module_name().contains('.') {
                    QualifiedTypeName::namespaced(
                        config.root_package().to_string(),
                        name.name.clone(),
                    )
                } else {
                    name.clone()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
