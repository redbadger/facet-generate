//! C#-specific settings, separate from the shared per-module configuration.

use std::collections::BTreeMap;

use crate::{
    generation::CodeGeneratorConfig,
    reflection::format::{Format, FormatHolder, Namespace, QualifiedTypeName},
};

use super::{CSharpCodeGenerator, naming};

/// C#-specific generation settings.
///
/// Pass the same settings to the installer or generator and to custom plugins
/// that need to qualify independently reflected types. The language-independent
/// [`CodeGeneratorConfig`] remains unchanged.
#[derive(Clone, Debug, Default)]
pub struct CSharpConfig {
    pub(crate) external_namespaces: BTreeMap<String, String>,
}

impl CSharpConfig {
    /// Map a Rust registry namespace to its full external CLR namespace.
    ///
    /// Preserves casing and escapes C# keywords. When used with an installer,
    /// the namespace must also have an external package entry. See
    /// [`Installer::external_namespace`](super::Installer::external_namespace)
    /// for validation and runtime requirements.
    #[must_use]
    pub fn external_namespace(mut self, namespace: &str, clr_namespace: &str) -> Self {
        self.external_namespaces
            .insert(namespace.to_string(), clr_namespace.to_string());
        self
    }

    /// Qualify a registry-spelled name using these C# namespace mappings.
    ///
    /// Like [`super::requalify`], apply exactly once. Plugin emission contexts
    /// already contain qualified formats; use this for independently reflected
    /// names or enum metadata queries. Without a mapping, uses the normal
    /// root-package rules.
    #[must_use]
    pub fn requalify(
        &self,
        config: &CodeGeneratorConfig,
        name: &QualifiedTypeName,
    ) -> QualifiedTypeName {
        if let Namespace::Named(namespace) = &name.namespace
            && let Some(external) = self.external_namespaces.get(namespace)
        {
            let external = external
                .split('.')
                .map(|segment| naming::RULES.escape(segment))
                .collect::<Vec<_>>()
                .join(".");
            return QualifiedTypeName::namespaced(format!("global::{external}"), name.name.clone());
        }
        CSharpCodeGenerator::requalify(config, name)
    }

    /// Qualify every type reference in a registry-spelled format.
    ///
    /// Apply exactly once, as with [`Self::requalify`].
    pub fn requalify_format(&self, config: &CodeGeneratorConfig, format: &mut Format) {
        self.requalify_type_names(config, format);
    }

    pub(super) fn requalify_type_names(
        &self,
        config: &CodeGeneratorConfig,
        holder: &mut impl FormatHolder,
    ) {
        let _ = holder.visit_mut(&mut |format| {
            if let Format::TypeName(name) = format {
                *name = self.requalify(config, name);
            }
            Ok(())
        });
    }
}
