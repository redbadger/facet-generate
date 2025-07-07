// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Module defining the Abstract Syntax Tree (AST) of Serde formats.
//!
//! Node of the AST are made of the following types:
//! * `ContainerFormat`: the format of a container (struct or enum),
//! * `Format`: the format of an unnamed value,
//! * `Named<Format>`: the format of a field in a struct,
//! * `VariantFormat`: the format of a variant in a enum,
//! * `Named<VariantFormat>`: the format of a variant in a enum, together with its name,
//! * `Variable<Format>`: a variable holding an initially unknown value format,
//! * `Variable<VariantFormat>`: a variable holding an initially unknown variant format.
#![allow(clippy::missing_errors_doc)]

#[cfg(test)]
mod tests;

use crate::{Result, error::Error};
use serde::{
    Deserialize, Serialize, de, ser,
    ser::{SerializeMap, SerializeStruct},
};
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::BTreeMap,
    rc::Rc,
};

/// Represents a namespace in the type system.
#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum Namespace {
    /// The root namespace (no namespace prefix).
    Root,
    /// A named namespace.
    Named(String),
}

/// A qualified type name with namespace information.
#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq, Hash, PartialOrd, Ord)]
pub struct QualifiedTypeName {
    /// The namespace containing this type.
    pub namespace: Namespace,
    /// The simple name of the type.
    pub name: String,
}

impl From<&str> for QualifiedTypeName {
    fn from(value: &str) -> Self {
        Self::from_legacy_string(value)
    }
}

impl QualifiedTypeName {
    /// Create a new qualified type name in the root namespace.
    #[must_use]
    pub fn root(name: String) -> Self {
        Self {
            namespace: Namespace::Root,
            name,
        }
    }

    /// Create a new qualified type name in a named namespace.
    #[must_use]
    pub fn namespaced(namespace: String, name: String) -> Self {
        Self {
            namespace: Namespace::Named(namespace),
            name,
        }
    }

    /// Convert to the legacy dot-separated string format for compatibility.
    #[must_use]
    pub fn to_legacy_string(&self, namespace_formatter: fn(&str) -> String) -> String {
        match &self.namespace {
            Namespace::Root => self.name.clone(),
            Namespace::Named(ns) => format!("{}.{}", namespace_formatter(ns), self.name),
        }
    }

    /// Parse from the legacy dot-separated string format.
    #[must_use]
    pub fn from_legacy_string(s: &str) -> Self {
        if let Some((namespace, name)) = s.split_once('.') {
            Self::namespaced(namespace.to_string(), name.to_string())
        } else {
            Self::root(s.to_string())
        }
    }
}

/// Serde-based serialization format for anonymous "value" types.
#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Format {
    /// A format whose value is initially unknown. Used internally for tracing. Not (de)serializable.
    Variable(#[serde(with = "not_implemented")] Variable<Format>),

    /// The name of a container.
    TypeName(QualifiedTypeName),

    // The formats of primitive types
    Unit,
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Char,
    Str,
    Bytes,

    /// The format of `Option<T>`.
    Option(Box<Format>),
    /// A sequence, e.g. the format of `Vec<Foo>`.
    Seq(Box<Format>),
    /// A map, e.g. the format of `BTreeMap<K, V>`.
    #[serde(rename_all = "UPPERCASE")]
    Map {
        key: Box<Format>,
        value: Box<Format>,
    },

    /// A tuple, e.g. the format of `(Foo, Bar)`.
    Tuple(Vec<Format>),
    /// Alias for `(Foo, ... Foo)`.
    /// E.g. the format of `[Foo; N]`.
    #[serde(rename_all = "UPPERCASE")]
    TupleArray {
        content: Box<Format>,
        size: usize,
    },
}

/// Serde-based serialization format for named "container" types.
/// In Rust, those are enums and structs.
#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ContainerFormat {
    /// An empty struct, e.g. `struct A`.
    UnitStruct,
    /// A struct with a single unnamed parameter, e.g. `struct A(u16)`
    NewTypeStruct(Box<Format>),
    /// A struct with several unnamed parameters, e.g. `struct A(u16, u32)`
    TupleStruct(Vec<Format>),
    /// A struct with named parameters, e.g. `struct A { a: Foo }`.
    Struct(Vec<Named<Format>>),
    /// An enum, that is, an enumeration of variants.
    /// Each variant has a unique name and index within the enum.
    Enum(BTreeMap<u32, Named<VariantFormat>>),
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
/// A named value.
/// Used for named parameters or variants.
pub struct Named<T> {
    pub name: String,
    pub value: T,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
/// A mutable holder for an initially unknown value.
pub struct Variable<T>(Rc<RefCell<Option<T>>>);

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
/// Description of a variant in an enum.
pub enum VariantFormat {
    /// A variant whose format is initially unknown. Used internally for tracing. Not (de)serializable.
    Variable(#[serde(with = "not_implemented")] Variable<VariantFormat>),
    /// A variant without parameters, e.g. `A` in `enum X { A }`
    Unit,
    /// A variant with a single unnamed parameter, e.g. `A` in `enum X { A(u16) }`
    NewType(Box<Format>),
    /// A struct with several unnamed parameters, e.g. `A` in `enum X { A(u16, u32) }`
    Tuple(Vec<Format>),
    /// A struct with named parameters, e.g. `A` in `enum X { A { a: Foo } }`
    Struct(Vec<Named<Format>>),
}

/// Common methods for nodes in the AST of formats.
pub trait FormatHolder {
    /// Visit all the formats in `self` in a depth-first way.
    /// Variables are not supported and will cause an error.
    fn visit<'a>(&'a self, f: &mut dyn FnMut(&'a Format) -> Result<()>) -> Result<()>;

    /// Mutably visit all the formats in `self` in a depth-first way.
    /// * Replace variables (if any) with their known values then apply the
    ///   visiting function `f`.
    /// * Return an error if any variable has still an unknown value (thus cannot be removed).
    fn visit_mut(&mut self, f: &mut dyn FnMut(&mut Format) -> Result<()>) -> Result<()>;

    /// Finalize the formats within `self` by removing variables and making sure
    /// that all eligible tuples are compressed into a `TupleArray`. Return an error
    /// if any variable has an unknown value.
    fn normalize(&mut self) -> Result<()> {
        self.visit_mut(&mut |format: &mut Format| {
            let normalized = match format {
                Format::Tuple(formats) => {
                    let size = formats.len();
                    if size <= 1 {
                        return Ok(());
                    }
                    let format0 = &formats[0];
                    for format in formats.iter().skip(1) {
                        if format != format0 {
                            return Ok(());
                        }
                    }
                    Format::TupleArray {
                        content: Box::new(std::mem::take(&mut formats[0])),
                        size,
                    }
                }
                _ => {
                    return Ok(());
                }
            };
            *format = normalized;
            Ok(())
        })
    }

    /// Attempt to remove known variables within `self`. Silently abort
    /// if some variables have unknown values.
    fn reduce(&mut self) {
        self.visit_mut(&mut |_| Ok(())).unwrap_or(());
    }

    /// Whether this format is a variable with no known value yet.
    fn is_unknown(&self) -> bool;
}

impl FormatHolder for VariantFormat {
    fn visit<'a>(&'a self, f: &mut dyn FnMut(&'a Format) -> Result<()>) -> Result<()> {
        match self {
            Self::Variable(variable) => variable.visit(f)?,
            Self::Unit => (),
            Self::NewType(format) => format.visit(f)?,
            Self::Tuple(formats) => {
                for format in formats {
                    format.visit(f)?;
                }
            }
            Self::Struct(named_formats) => {
                for format in named_formats {
                    format.visit(f)?;
                }
            }
        }
        Ok(())
    }

    fn visit_mut(&mut self, f: &mut dyn FnMut(&mut Format) -> Result<()>) -> Result<()> {
        match self {
            Self::Variable(variable) => {
                variable.visit_mut(f)?;
                // At this point, `variable` is known and points to variable-free content.
                // Remove the variable.
                *self = std::mem::take(variable)
                    .into_inner()
                    .expect("variable is known");
            }
            Self::Unit => (),
            Self::NewType(format) => {
                format.visit_mut(f)?;
            }
            Self::Tuple(formats) => {
                for format in formats {
                    format.visit_mut(f)?;
                }
            }
            Self::Struct(named_formats) => {
                for format in named_formats {
                    format.visit_mut(f)?;
                }
            }
        }
        Ok(())
    }

    fn is_unknown(&self) -> bool {
        if let Self::Variable(v) = self {
            return v.is_unknown();
        }
        false
    }
}

impl<T> FormatHolder for Named<T>
where
    T: FormatHolder + std::fmt::Debug,
{
    fn visit<'a>(&'a self, f: &mut dyn FnMut(&'a Format) -> Result<()>) -> Result<()> {
        self.value.visit(f)
    }

    fn visit_mut(&mut self, f: &mut dyn FnMut(&mut Format) -> Result<()>) -> Result<()> {
        self.value.visit_mut(f)
    }

    fn is_unknown(&self) -> bool {
        false
    }
}

impl<T> Variable<T> {
    pub(crate) fn new(content: Option<T>) -> Self {
        Self(Rc::new(RefCell::new(content)))
    }

    #[must_use]
    pub fn borrow(&self) -> Ref<Option<T>> {
        self.0.as_ref().borrow()
    }

    #[must_use]
    pub fn borrow_mut(&self) -> RefMut<Option<T>> {
        self.0.as_ref().borrow_mut()
    }
}

impl<T> Variable<T>
where
    T: Clone,
{
    fn into_inner(self) -> Option<T> {
        match Rc::try_unwrap(self.0) {
            Ok(cell) => cell.into_inner(),
            Err(rc) => rc.borrow().clone(),
        }
    }
}

mod not_implemented {
    pub fn serialize<T, S>(_: &T, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        use serde::ser::Error;
        Err(S::Error::custom("Cannot serialize variables"))
    }

    pub fn deserialize<'de, T, D>(_deserializer: D) -> Result<T, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        use serde::de::Error;
        Err(D::Error::custom("Cannot deserialize variables"))
    }
}

impl<T> FormatHolder for Variable<T>
where
    T: FormatHolder + std::fmt::Debug + Clone,
{
    fn visit<'a>(&'a self, _f: &mut dyn FnMut(&'a Format) -> Result<()>) -> Result<()> {
        Err(Error::UnknownFormat)
    }

    fn visit_mut(&mut self, f: &mut dyn FnMut(&mut Format) -> Result<()>) -> Result<()> {
        match &mut *self.borrow_mut() {
            None => Err(Error::UnknownFormat),
            Some(value) => value.visit_mut(f),
        }
    }

    fn is_unknown(&self) -> bool {
        match self.borrow().as_ref() {
            None => true,
            Some(format) => format.is_unknown(),
        }
    }
}

impl FormatHolder for ContainerFormat {
    fn visit<'a>(&'a self, f: &mut dyn FnMut(&'a Format) -> Result<()>) -> Result<()> {
        match self {
            Self::UnitStruct => (),
            Self::NewTypeStruct(format) => format.visit(f)?,
            Self::TupleStruct(formats) => {
                for format in formats {
                    format.visit(f)?;
                }
            }
            Self::Struct(named_formats) => {
                for format in named_formats {
                    format.visit(f)?;
                }
            }
            Self::Enum(variants) => {
                for variant in variants {
                    variant.1.visit(f)?;
                }
            }
        }
        Ok(())
    }

    fn visit_mut(&mut self, f: &mut dyn FnMut(&mut Format) -> Result<()>) -> Result<()> {
        match self {
            Self::UnitStruct => (),
            Self::NewTypeStruct(format) => format.visit_mut(f)?,
            Self::TupleStruct(formats) => {
                for format in formats {
                    format.visit_mut(f)?;
                }
            }
            Self::Struct(named_formats) => {
                for format in named_formats {
                    format.visit_mut(f)?;
                }
            }
            Self::Enum(variants) => {
                for variant in variants {
                    variant.1.visit_mut(f)?;
                }
            }
        }
        Ok(())
    }

    fn is_unknown(&self) -> bool {
        false
    }
}

impl FormatHolder for Format {
    fn visit<'a>(&'a self, f: &mut dyn FnMut(&'a Format) -> Result<()>) -> Result<()> {
        match self {
            Self::Variable(variable) => variable.visit(f)?,
            Self::TypeName(_)
            | Self::Unit
            | Self::Bool
            | Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
            | Self::I128
            | Self::U8
            | Self::U16
            | Self::U32
            | Self::U64
            | Self::U128
            | Self::F32
            | Self::F64
            | Self::Char
            | Self::Str
            | Self::Bytes => (),

            Self::Option(format)
            | Self::Seq(format)
            | Self::TupleArray {
                content: format, ..
            } => {
                format.visit(f)?;
            }

            Self::Map { key, value } => {
                key.visit(f)?;
                value.visit(f)?;
            }

            Self::Tuple(formats) => {
                for format in formats {
                    format.visit(f)?;
                }
            }
        }
        f(self)
    }

    fn visit_mut(&mut self, f: &mut dyn FnMut(&mut Format) -> Result<()>) -> Result<()> {
        match self {
            Self::Variable(variable) => {
                variable.visit_mut(f)?;
                // At this point, `variable` is known and points to variable-free content.
                // Remove the variable.
                *self = std::mem::take(variable)
                    .into_inner()
                    .expect("variable is known");
            }
            Self::TypeName(_)
            | Self::Unit
            | Self::Bool
            | Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
            | Self::I128
            | Self::U8
            | Self::U16
            | Self::U32
            | Self::U64
            | Self::U128
            | Self::F32
            | Self::F64
            | Self::Char
            | Self::Str
            | Self::Bytes => (),

            Self::Option(format)
            | Self::Seq(format)
            | Self::TupleArray {
                content: format, ..
            } => {
                format.visit_mut(f)?;
            }

            Self::Map { key, value } => {
                key.visit_mut(f)?;
                value.visit_mut(f)?;
            }

            Self::Tuple(formats) => {
                for format in formats {
                    format.visit_mut(f)?;
                }
            }
        }
        f(self)
    }

    fn is_unknown(&self) -> bool {
        if let Self::Variable(v) = self {
            return v.is_unknown();
        }
        false
    }
}

impl Format {
    /// Return a format made of a fresh variable with no known value.
    #[must_use]
    pub fn unknown() -> Self {
        Self::Variable(Variable::new(None))
    }
}

impl VariantFormat {
    /// Return a format made of a fresh variable with no known value.
    #[must_use]
    pub fn unknown() -> Self {
        Self::Variable(Variable::new(None))
    }
}

impl Default for Format {
    fn default() -> Self {
        Self::unknown()
    }
}

impl Default for VariantFormat {
    fn default() -> Self {
        Self::unknown()
    }
}

// For better rendering in human readable formats, we wish to serialize
// `Named { key: x, value: y }` as a map `{ x: y }`.
impl<T> Serialize for Named<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if serializer.is_human_readable() {
            let mut map = serializer.serialize_map(Some(1))?;
            map.serialize_entry(&self.name, &self.value)?;
            map.end()
        } else {
            let mut inner = serializer.serialize_struct("Named", 2)?;
            inner.serialize_field("name", &self.name)?;
            inner.serialize_field("value", &self.value)?;
            inner.end()
        }
    }
}

struct NamedVisitor<T> {
    marker: std::marker::PhantomData<T>,
}

impl<T> NamedVisitor<T> {
    fn new() -> Self {
        Self {
            marker: std::marker::PhantomData,
        }
    }
}

impl<'de, T> de::Visitor<'de> for NamedVisitor<T>
where
    T: Deserialize<'de>,
{
    type Value = Named<T>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a single entry map")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: de::MapAccess<'de>,
    {
        let named_value = match access.next_entry::<String, T>()? {
            Some((name, value)) => Named { name, value },
            _ => {
                return Err(de::Error::custom("Missing entry"));
            }
        };
        if access.next_entry::<String, T>()?.is_some() {
            return Err(de::Error::custom("Too many entries"));
        }
        Ok(named_value)
    }
}

/// For deserialization of non-human readable `Named` values, we keep it simple and use derive macros.
#[derive(Deserialize)]
#[serde(rename = "Named")]
struct NamedInternal<T> {
    name: String,
    value: T,
}

impl<'de, T> Deserialize<'de> for Named<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Named<T>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            deserializer.deserialize_map(NamedVisitor::new())
        } else {
            let NamedInternal { name, value } = NamedInternal::deserialize(deserializer)?;
            Ok(Self { name, value })
        }
    }
}
