use std::{cmp::Ordering, collections::BTreeMap};

use serde::Serialize;

use crate::{
    Result,
    generation::CodeGeneratorConfig,
    reflection::{
        Registry,
        format::{ContainerFormat, Format, FormatHolder, Namespace},
    },
};

#[derive(Debug, Clone, Serialize)]
pub struct Module(CodeGeneratorConfig);

impl Module {
    #[must_use]
    pub fn new(module_name: String) -> Self {
        Module(CodeGeneratorConfig::new(module_name))
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

/// Splits a registry by namespace.
#[must_use]
pub fn split(root: &str, registry: Registry) -> BTreeMap<Module, Registry> {
    let mut registries = BTreeMap::<Module, Registry>::new();
    for (name, mut format) in registry {
        registries
            .entry(
                make_module(root, &mut format, &name.namespace)
                    .expect("should not have any remaining placeholders"),
            )
            .or_default()
            .insert(name, format.clone());
    }
    registries
}

fn make_module(root: &str, format: &mut ContainerFormat, namespace: &Namespace) -> Result<Module> {
    let mut external_definitions: BTreeMap<String, Vec<String>> = BTreeMap::new();
    format.visit(&mut |format| {
        if let Format::TypeName(qualified_name) = format {
            if let Namespace::Named(ns) = &qualified_name.namespace {
                external_definitions
                    .entry(ns.to_string())
                    .or_default()
                    .push(qualified_name.name.clone());
            }
        }
        Ok(())
    })?;
    let namespace = match namespace {
        Namespace::Root => root,
        Namespace::Named(ns) => ns,
    };

    let config = CodeGeneratorConfig::new(namespace.to_string())
        .with_external_definitions(external_definitions);

    Ok(Module(config))
}

#[cfg(test)]
mod tests;
