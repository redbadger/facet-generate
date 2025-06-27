use std::{cmp::Ordering, collections::BTreeMap};

use serde::Serialize;

use crate::{
    serde_generate::CodeGeneratorConfig,
    serde_reflection::{ContainerFormat, Format, FormatHolder, Registry, Result},
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

pub fn split(root: &str, registry: Registry) -> Result<BTreeMap<Namespace, Registry>> {
    let mut registries = BTreeMap::<Namespace, Registry>::new();
    for (name, mut format) in registry {
        if let Some((namespace, name)) = name.split_once('.') {
            registries
                .entry(make_namespace(&mut format, namespace)?)
                .or_default()
                .insert(name.to_string(), format.clone());
        } else {
            registries
                .entry(make_namespace(&mut format, root)?)
                .or_default()
                .insert(name.to_string(), format.clone());
        }
    }
    Ok(registries)
}

fn make_namespace(format: &mut ContainerFormat, namespace: &str) -> Result<Namespace> {
    let mut external_definitions: BTreeMap<String, Vec<String>> = BTreeMap::new();
    format.visit_mut(&mut |format| {
        if let Format::TypeName(name) = format {
            if let Some((namespace, name)) = name.split_once('.') {
                external_definitions
                    .entry(namespace.to_string())
                    .or_default()
                    .push(name.to_string());
                *format = Format::TypeName(name.to_string());
            }
        }
        Ok(())
    })?;
    let config = CodeGeneratorConfig::new(namespace.to_string())
        .with_external_definitions(external_definitions);

    Ok(Namespace(config))
}

#[cfg(test)]
mod tests;
