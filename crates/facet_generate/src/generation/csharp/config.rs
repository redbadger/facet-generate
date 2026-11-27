//! C#-specific settings, separate from the shared per-module configuration.

use std::collections::BTreeMap;

use crate::{
    Registry,
    generation::CodeGeneratorConfig,
    reflection::format::{Format, FormatHolder, Namespace, QualifiedTypeName},
};

use super::{CSharpCodeGenerator, naming};

/// C#-specific generation settings.
///
/// Give these settings to [`CSharpCodeGenerator::with_csharp_config`]. A
/// custom plugin that qualifies types that it reflects itself needs the same
/// settings. Give the plugin a clone. The [`Installer`](super::Installer) has
/// no `CSharpConfig`. Set its mappings with
/// [`Installer::external_namespace`](super::Installer::external_namespace).
/// The language-independent [`CodeGeneratorConfig`] does not change.
#[derive(Clone, Debug, Default)]
pub struct CSharpConfig {
    pub(crate) external_namespaces: BTreeMap<String, String>,
}

impl CSharpConfig {
    /// Map a Rust registry namespace to its full external CLR namespace.
    ///
    /// The generated code keeps the casing of `clr_namespace` and escapes C#
    /// keywords. The generator checks each mapping before it writes output.
    /// See [`Installer::external_namespace`](super::Installer::external_namespace)
    /// for the rules that a mapping must obey.
    #[must_use]
    pub fn external_namespace(mut self, namespace: &str, clr_namespace: &str) -> Self {
        self.external_namespaces
            .insert(namespace.to_string(), clr_namespace.to_string());
        self
    }

    /// Checks that each mapping names a valid CLR namespace.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidInput`](std::io::ErrorKind::InvalidInput) error if
    /// a CLR namespace has an empty segment, an alias qualifier such as
    /// `global::`, or a segment that is not a plain CLR identifier.
    pub(crate) fn check_spelling(&self) -> std::io::Result<()> {
        for (namespace, clr_namespace) in &self.external_namespaces {
            let problem = if clr_namespace.split('.').any(str::is_empty) {
                "requires a nonempty dotted CLR namespace"
            } else if clr_namespace.contains("::") {
                "must name a CLR namespace without an alias qualifier"
            } else if !clr_namespace.split('.').all(naming::plain_identifier) {
                "requires plain CLR identifiers without source escapes, formatting characters or whitespace"
            } else {
                continue;
            };
            return Err(mapping_error(namespace, problem));
        }
        Ok(())
    }

    /// Checks these mappings against the module that `config` and `registry`
    /// generate.
    ///
    /// # Errors
    ///
    /// Returns an [`InvalidInput`](std::io::ErrorKind::InvalidInput) error if
    /// a mapping is not valid (see [`Self::check_spelling`]). It also returns
    /// one if the module is a mapped namespace, or declares a type in one.
    /// References to such a type would go to the external CLR namespace, not
    /// to the declaration.
    pub(crate) fn check(
        &self,
        config: &CodeGeneratorConfig,
        registry: &Registry,
    ) -> std::io::Result<()> {
        self.check_spelling()?;
        let declared = |namespace: &String| {
            config.is_own_namespace(namespace)
                || registry
                    .keys()
                    .any(|name| matches!(&name.namespace, Namespace::Named(n) if n == namespace))
        };
        if let Some(namespace) = self.external_namespaces.keys().find(|n| declared(n)) {
            return Err(mapping_error(
                namespace,
                "maps a namespace that this module generates. \
                 Remove the mapping, or generate the namespace in a separate package",
            ));
        }
        Ok(())
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

/// The error for the mapping of `namespace`, which has `problem`.
fn mapping_error(namespace: &str, problem: &str) -> std::io::Error {
    std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("C#: external namespace mapping for \"{namespace}\" {problem}"),
    )
}
