use std::{cmp::Ordering, collections::BTreeMap};

use serde::Serialize;

use crate::{
    generation::CodeGeneratorConfig,
    reflection::{ContainerFormat, Format, FormatHolder, Registry, format},
};

#[derive(Debug, Clone, Serialize)]
pub struct Namespace(CodeGeneratorConfig);

impl Namespace {
    #[must_use]
    pub fn new(module_name: String) -> Self {
        Namespace(CodeGeneratorConfig::new(module_name))
    }

    #[must_use]
    pub fn config(&self) -> &CodeGeneratorConfig {
        &self.0
    }
}

impl std::hash::Hash for Namespace {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.module_name.hash(state);
    }
}

impl Eq for Namespace {}

impl PartialEq for Namespace {
    fn eq(&self, other: &Self) -> bool {
        self.0.module_name == other.0.module_name
    }
}

impl PartialOrd for Namespace {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Namespace {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.module_name.cmp(&other.0.module_name)
    }
}

/// Splits a registry by namespace.
#[must_use]
pub fn split(root: &str, registry: Registry) -> BTreeMap<Namespace, Registry> {
    let mut registries = BTreeMap::<Namespace, Registry>::new();
    for (name, mut format) in registry {
        registries
            .entry(
                make_namespace(root, &mut format, &name.namespace)
                    .expect("should not have any remaining placeholders"),
            )
            .or_default()
            .insert(name, format.clone());
    }
    registries
}

fn make_namespace(
    root: &str,
    format: &mut ContainerFormat,
    namespace: &format::Namespace,
) -> super::Result<Namespace> {
    let mut external_definitions: BTreeMap<String, Vec<String>> = BTreeMap::new();
    format.visit(&mut |format| {
        if let Format::TypeName(qualified_name) = format {
            if let format::Namespace::Named(ns) = &qualified_name.namespace {
                external_definitions
                    .entry(ns.to_string())
                    .or_default()
                    .push(qualified_name.name.clone());
            }
        }
        Ok(())
    })?;
    let namespace = match namespace {
        format::Namespace::Root => root,
        format::Namespace::Named(ns) => ns,
    };

    let config = CodeGeneratorConfig::new(namespace.to_string())
        .with_external_definitions(external_definitions);

    Ok(Namespace(config))
}

#[cfg(test)]
mod tests;
