use std::io::{Result, Write};

use crate::{
    Registry,
    generation::{
        CodeGeneratorConfig, Emitter, Language,
        indent::{IndentConfig, IndentedWriter},
        module::Module,
    },
};

/// Main configuration object for code-generation in Kotlin
pub struct CodeGenerator<'a> {
    /// Language-independent configuration
    pub(crate) config: &'a CodeGeneratorConfig,
}

impl<'a> Language<'a> for CodeGenerator<'a> {
    fn new(config: &'a CodeGeneratorConfig) -> Self {
        CodeGenerator::new(config)
    }

    fn write_output<W: std::io::Write>(
        &mut self,
        writer: &mut W,
        registry: &Registry,
    ) -> Result<()> {
        self.output(writer, registry)
    }
}

impl<'a> CodeGenerator<'a> {
    /// Create a Kotlin code generator for the given config
    #[must_use]
    pub fn new(config: &'a CodeGeneratorConfig) -> Self {
        Self { config }
    }

    /// Output type definitions for `registry`.
    /// # Errors
    /// This function may fail if the writer encounters an error while writing the generated code.
    pub fn output(&self, out: &mut dyn Write, registry: &Registry) -> Result<()> {
        let w = &mut IndentedWriter::new(out, IndentConfig::Space(4));

        let module = Module::new(self.config.module_name.clone());
        module.write(w)?;

        for (i, container) in registry.iter().enumerate() {
            if i > 0 {
                writeln!(w)?;
            }
            container.write(w)?;
        }

        Ok(())
    }
}
