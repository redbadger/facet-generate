use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::reflection::format::{ContainerFormat, QualifiedTypeName};

/// A map of container formats.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Registry {
    pub containers: BTreeMap<QualifiedTypeName, ContainerFormat>,
}
