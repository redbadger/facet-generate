//! Splits a [`Registry`] into per-namespace modules for multi-file output.
//!
//! Types in the registry are grouped by their [`Namespace`]. Each group
//! becomes a [`Module`] with its own [`CodeGeneratorConfig`] and sub-registry.
//! Cross-namespace type references are tracked as external definitions so that
//! generators can emit the correct import statements.
//!
//! The entry point is [`split`], which returns a `BTreeMap<Module, Registry>`
//! — one entry per namespace, ordered by module name.

use std::{cmp::Ordering, collections::BTreeMap};

use serde::Serialize;

use crate::{
    Registry,
    generation::CodeGeneratorConfig,
    reflection::format::{ContainerFormat, Format, FormatHolder, Namespace, QualifiedTypeName},
};

/// A single output module, identified by namespace name and carrying the
/// [`CodeGeneratorConfig`] for that namespace.
///
/// Equality, ordering, and hashing are based on the module name alone, so
/// modules can be used as `BTreeMap` keys.
#[derive(Debug, Clone, Serialize)]
pub struct Module(CodeGeneratorConfig);

impl Module {
    #[must_use]
    pub fn new(config: &CodeGeneratorConfig) -> Self {
        Module(config.clone())
    }

    #[must_use]
    pub fn config(&self) -> &CodeGeneratorConfig {
        &self.0
    }
}

impl std::hash::Hash for Module {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.module_name.hash(state);
    }
}

impl Eq for Module {}

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        self.0.module_name == other.0.module_name
    }
}

impl PartialOrd for Module {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Module {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.module_name.cmp(&other.0.module_name)
    }
}

/// Splits a registry into one [`Module`] per namespace.
///
/// Types with [`Namespace::Root`] are assigned to the `root` module name.
/// For each module, any `Format::TypeName` references that point to a
/// *different* namespace are recorded as external definitions so generators
/// can emit import statements.
///
/// # Panics
///
/// Panics if the registry contains unresolved format placeholders.
#[must_use]
pub fn split(root: &str, registry: &Registry) -> BTreeMap<Module, Registry> {
    // First, group types by their target namespace
    let mut namespace_groups = BTreeMap::<String, Vec<(QualifiedTypeName, ContainerFormat)>>::new();

    for (name, format) in registry {
        let namespace_key = match &name.namespace {
            Namespace::Root => root.to_string(),
            Namespace::Named(ns) => ns.clone(),
        };
        namespace_groups
            .entry(namespace_key)
            .or_default()
            .push((name.clone(), format.clone()));
    }

    // Then create one module per namespace, collecting all external dependencies
    let mut registries = BTreeMap::<Module, Registry>::new();

    for (namespace_key, types) in namespace_groups {
        // Collect all external dependencies for this namespace
        let mut all_external_definitions: BTreeMap<String, Vec<String>> = BTreeMap::new();

        for (_, format) in &types {
            let format_clone = format.clone();
            format_clone
                .visit(&mut |format| {
                    if let Format::TypeName(qualified_name) = format
                        && let Namespace::Named(ns) = &qualified_name.namespace
                        && ns != &namespace_key
                    {
                        all_external_definitions
                            .entry(ns.clone())
                            .or_default()
                            .push(qualified_name.name.clone());
                    }
                    Ok(())
                })
                .expect("should not have any remaining placeholders");
        }

        // Create the module with all collected external dependencies
        let config = CodeGeneratorConfig::new(namespace_key)
            .with_external_definitions(all_external_definitions);
        let module = Module(config);

        // Add all types to this module's registry
        let mut module_registry = Registry::new();
        for (name, format) in types {
            module_registry.insert(name, format);
        }

        registries.insert(module, module_registry);
    }

    registries
}

#[cfg(test)]
#[path = "./module_tests.rs"]
mod module_tests;
