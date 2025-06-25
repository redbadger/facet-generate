use std::collections::BTreeMap;

use serde::Serialize;

use crate::serde_reflection::Registry;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize)]
pub enum Namespace {
    Root,
    Child(String),
}

#[must_use]
pub fn split(registry: &Registry) -> BTreeMap<Namespace, Registry> {
    let mut registries = BTreeMap::<Namespace, Registry>::new();
    for (name, format) in registry {
        if let Some((namespace, name)) = name.split_once('.') {
            registries
                .entry(Namespace::Child(namespace.to_string()))
                .or_default()
                .insert(name.to_string(), format.clone());
        } else {
            registries
                .entry(Namespace::Root)
                .or_default()
                .insert(name.to_string(), format.clone());
        }
    }
    registries
}

#[cfg(test)]
mod tests;
