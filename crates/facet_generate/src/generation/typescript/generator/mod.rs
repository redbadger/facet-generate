//! Top-level orchestrator for TypeScript code generation.
//!
//! [`TypeScriptCodeGenerator`] implements [`CodeGenerator`] and is the entry point for
//! producing a single TypeScript source file from a [`Registry`]. It carries
//! It delegates writing to the emitter layer.

use std::{
    io::{Result, Write},
    sync::Arc,
};

use crate::{
    Registry,
    generation::{
        CodeGenerator, CodeGeneratorConfig, Container, Emitter, Encoding, bincode::BincodePlugin,
        indent::IndentedWriter, json::JsonPlugin, module::Module, typescript::emitter::TypeScript,
    },
    reflection::format::{Format, FormatHolder, Namespace, QualifiedTypeName},
};

/// Main configuration object for TypeScript code generation.
///
/// Wraps a [`CodeGeneratorConfig`] and implements [`CodeGenerator`] so it
/// can be used by the installer pipeline.
pub struct TypeScriptCodeGenerator<'a> {
    /// Language-independent configuration.
    pub(crate) config: &'a CodeGeneratorConfig,
    /// Which serialization encoding to generate code for.
    pub(crate) encoding: Encoding,
}

impl<'a> CodeGenerator<'a> for TypeScriptCodeGenerator<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        TypeScriptCodeGenerator::new(config)
    }

    fn write_output<W: Write>(&mut self, writer: &mut W, registry: &Registry) -> Result<()> {
        self.output(writer, registry)
    }
}

impl<'a> TypeScriptCodeGenerator<'a> {
    /// Create a TypeScript code generator with no encoding (plain types only).
    ///
    /// Call [`with_encoding`](Self::with_encoding) to enable serialization.
    #[must_use]
    pub const fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self {
            config,
            encoding: Encoding::None,
        }
    }

    /// Set the encoding, returning the modified generator.
    #[must_use]
    pub const fn with_encoding(mut self, encoding: Encoding) -> Self {
        self.encoding = encoding;
        self
    }

    /// Produce a complete TypeScript source file for the types in `registry`.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `out` fails.
    pub fn output(&self, out: &mut impl Write, registry: &Registry) -> Result<()> {
        let w = &mut IndentedWriter::new(out, self.config.indent);

        let mut config = self.config.clone();
        config.update_from(registry);

        let lang = {
            let base = TypeScript::new(&config, registry);
            match self.encoding {
                Encoding::Bincode => base.with_plugin(Arc::new(BincodePlugin)),
                Encoding::Json => base.with_plugin(Arc::new(JsonPlugin)),
                Encoding::None => base,
            }
        };

        Module::new(&config).write(w, &lang)?;

        let updated_registry = Self::update_qualified_names(&config, registry);
        for container in updated_registry.iter().map(Container::from) {
            container.write(w, &lang)?;
        }

        Ok(())
    }

    /// Updates [`QualifiedTypeName`] instances for TypeScript's ES-module
    /// namespacing:
    ///
    /// 1. **Same-module type** — strip namespace to `Root` so it renders as a
    ///    bare name (e.g. `Child`).
    /// 2. **External type in different namespace** — keep its `Named` namespace,
    ///    which renders as `Namespace.Type` (e.g. `Other.Child`) via the
    ///    wildcard import added by the [`Module`](super::super::module::Module)
    ///    emitter.
    fn update_qualified_names(config: &CodeGeneratorConfig, registry: &Registry) -> Registry {
        let mut updated_registry = registry.clone();

        for container_format in updated_registry.values_mut() {
            let _ = container_format.visit_mut(&mut |format| {
                if let Format::TypeName(qualified_name) = format
                    && let Namespace::Named(namespace) = &qualified_name.namespace
                    && namespace == config.module_name()
                {
                    // Same-module type: strip namespace so it renders as a bare name
                    *qualified_name = QualifiedTypeName::root(qualified_name.name.clone());
                }
                Ok(())
            });
        }

        updated_registry
    }
}

#[cfg(test)]
mod tests;
