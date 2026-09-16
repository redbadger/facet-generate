#![expect(unused)]

use std::collections::{BTreeMap, HashSet};

use facet::Facet;

use crate as fg;

// The crux key-value shape: each operation is a top-level struct, so `Set` is
// a declared type in the generated module while `Set<T>` is still needed for
// the `HashSet` field on `Store`.
#[derive(Facet)]
pub struct Get {
    pub key: String,
}

#[derive(Facet)]
pub struct Set {
    pub key: String,
    #[facet(fg::bytes)]
    pub value: Vec<u8>,
}

#[derive(Facet)]
pub struct Delete {
    pub key: String,
}

#[derive(Facet)]
pub struct Exists {
    pub key: String,
}

#[derive(Facet)]
pub struct ListKeys {
    pub prefix: String,
    pub cursor: u64,
}

#[derive(Facet)]
pub struct Keys {
    // Named `items` rather than `keys`: a C# property may not share its name
    // with its enclosing type (CS0542), which the pre-pass rejects.
    pub items: Vec<String>,
    pub next_cursor: u64,
}

#[derive(Facet)]
#[repr(C)]
pub enum ValueResult {
    Ok(Option<Vec<u8>>),
    Err(String),
}

#[derive(Facet)]
#[repr(C)]
pub enum BoolResult {
    Ok(bool),
    Err(String),
}

#[derive(Facet)]
#[repr(C)]
pub enum KeysResult {
    Ok(Keys),
    Err(String),
}

// Exercises every container the emitters write with a bare builtin name:
// `Set`, `List`/`Seq`, `Map`/`Dictionary`, `String` and `Pair`. Only `Set` is
// shadowed by a declaration, so only `Set` is written qualified.
#[derive(Facet)]
pub struct Store {
    pub tags: HashSet<String>,
    pub entries: BTreeMap<String, String>,
    pub blob: Vec<u8>,
    pub pair: (i32, String),
}

crate::test! {
    Get, Set, Delete, Exists, ListKeys, Keys, ValueResult, BoolResult, KeysResult, Store
    for kotlin, swift, typescript, csharp
}
