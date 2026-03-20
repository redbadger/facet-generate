// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The format AST — a language-neutral representation of type shapes for code generation.
//!
//! The [`RegistryBuilder`](super::RegistryBuilder) populates these nodes by walking `facet`
//! type metadata; code generators consume them to emit equivalent types in target languages.
//!
//! The key types form a hierarchy:
//!
//! - [`ContainerFormat`] — a named top-level type: unit struct, newtype, struct with fields,
//!   or enum with variants. These are the values in the [`Registry`](crate::Registry).
//! - [`Format`] — the shape of a value: primitives (`Bool`, `U32`, `Str`, ...),
//!   composites (`Option`, `Seq`, `Map`, `Tuple`), or a reference to another container
//!   via `TypeName`.
//! - [`VariantFormat`] — the payload shape of an enum variant: `Unit`, `NewType`, `Tuple`,
//!   or `Struct`.
//! - [`Named<T>`] — wraps a `Format` or `VariantFormat` with a name and doc comments,
//!   used for struct fields and enum variants.
//! - [`QualifiedTypeName`] — a type name qualified by a [`Namespace`] (root or named),
//!   used as the registry key.
#![allow(clippy::missing_errors_doc)]

#[cfg(test)]
mod tests;

use crate::{Result, error::Error};
use facet::{Field, Shape, Variant};
use serde::{
    Deserialize, Serialize, de, ser,
    ser::{SerializeMap, SerializeStruct},
};
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::BTreeMap,
    fmt,
    rc::Rc,
};

/// Controls how types are grouped in generated code.
///
/// Types in [`Root`](Namespace::Root) appear at the top level. Types in a
/// [`Named`](Namespace::Named) namespace are grouped together — how this manifests depends
/// on the target language (e.g. nested objects in Kotlin, namespaces in C#). Languages
/// without namespaces (e.g. Swift) flatten everything to the top level.
#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "UPPERCASE")]
pub enum Namespace {
    /// The root namespace (no namespace prefix).
    Root,
    /// A named namespace.
    Named(String),
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Namespace::Root => write!(f, "ROOT"),
            Namespace::Named(name) => write!(f, "{name}"),
        }
    }
}

/// The key used to identify types in the [`Registry`](crate::Registry) — a type name
/// paired with its [`Namespace`].
///
/// For example, a type `User` in namespace `Api` would be
/// `QualifiedTypeName { namespace: Named("Api"), name: "User" }`, displayed as `Api::User`.
#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq, Hash, PartialOrd, Ord)]
pub struct QualifiedTypeName {
    /// The namespace containing this type.
    pub namespace: Namespace,
    /// The simple name of the type.
    pub name: String,
}

impl fmt::Display for QualifiedTypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}", self.namespace, self.name)
    }
}

impl From<&str> for QualifiedTypeName {
    fn from(value: &str) -> Self {
        if let Some((namespace, name)) = value.split_once(['.', '_']) {
            Self::namespaced(namespace.to_string(), name.to_string())
        } else {
            Self::root(value.to_string())
        }
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

    /// Build a string from the qualified type name using the supplied namespace formatter and separator.
    #[must_use]
    pub fn format(&self, namespace_formatter: fn(&str) -> String, separator: &str) -> String {
        match &self.namespace {
            Namespace::Root => self.name.clone(),
            Namespace::Named(ns) => {
                let namespace = namespace_formatter(ns);
                let name = self.name.clone();
                format!("{namespace}{separator}{name}")
            }
        }
    }
}

/// Documentation comments extracted from Rust source types, carried through to code generation.
///
/// Each entry is a single line of a doc comment. Code generators use these to emit equivalent
/// documentation in the target language (e.g. `///` in C#, `/** */` in Kotlin).
#[derive(Serialize, Deserialize, Default, Debug, Eq, Clone, PartialEq)]
#[serde(transparent)]
pub struct Doc(Vec<String>);

impl Doc {
    #[must_use]
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, comment: String) {
        self.0.push(comment);
    }

    #[must_use]
    pub fn comments(&self) -> &[String] {
        &self.0
    }
}

impl From<&[&str]> for Doc {
    fn from(comments: &[&str]) -> Self {
        let doc = comments.iter().map(|c| c.trim().to_string()).collect();
        Self(doc)
    }
}

impl From<&Shape> for Doc {
    fn from(shape: &Shape) -> Self {
        shape.doc.into()
    }
}

impl From<&Field> for Doc {
    fn from(field: &Field) -> Self {
        field.doc.into()
    }
}

impl From<&Variant> for Doc {
    fn from(variant: &Variant) -> Self {
        variant.doc.into()
    }
}

/// The shape of a value type — an AST node describing how a field, variant payload, or nested
/// value is serialized. Primitives are leaf nodes; composites (`Option`, `Seq`, `Map`, `Tuple`)
/// contain nested `Format` nodes; and `TypeName` is a reference to a [`ContainerFormat`] in
/// the [`Registry`](crate::Registry).
///
/// `Format` and [`ContainerFormat`] are separate types, not variants of the same enum.
/// A `ContainerFormat` is a type declaration (a struct or enum definition) that holds `Format`
/// nodes as its children (field types, variant payloads). A `Format` is a type usage — and
/// `Format::TypeName` closes the loop by referencing back to a `ContainerFormat` in the registry.
#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Format {
    /// A placeholder for a format not yet known at construction time. `Format::unknown()` creates
    /// one of these, and it is also the `Default` value for `Format`. Fields start as variables
    /// and are filled in as the [`RegistryBuilder`](super::RegistryBuilder) processes each type,
    /// resolving to a concrete format (e.g. `U32`, `Str`, `TypeName(...)`). Not involved in code
    /// generation — all variables must be resolved before
    /// [`RegistryBuilder::build`](super::RegistryBuilder::build) completes.
    Variable(#[serde(with = "not_implemented")] Variable<Format>),

    /// A reference to a named container type in the [`Registry`](crate::Registry).
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
    /// A set, e.g. the format of `HashSet<Foo>`.
    Set(Box<Format>),
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

/// The shape of a named top-level type — a struct or enum that gets its own entry in the
/// [`Registry`](crate::Registry). Each variant holds the [`Format`] nodes that describe
/// its fields or inner values, plus a [`Doc`] for documentation comments.
///
/// See [`Format`] for how these are referenced from within other containers.
#[derive(Serialize, Deserialize, Debug, Eq, Clone, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ContainerFormat {
    /// An empty struct, e.g. `struct A`.
    UnitStruct(Doc),
    /// A struct with a single unnamed parameter, e.g. `struct A(u16)`
    NewTypeStruct(Box<Format>, Doc),
    /// A struct with several unnamed parameters, e.g. `struct A(u16, u32)`
    TupleStruct(Vec<Format>, Doc),
    /// A struct with named parameters, e.g. `struct A { a: Foo }`.
    Struct(Vec<Named<Format>>, Doc),
    /// An enum, that is, an enumeration of variants.
    /// Each variant has a unique name and index within the enum.
    Enum(BTreeMap<u32, Named<VariantFormat>>, Doc),
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
/// Attaches a name and [`Doc`] comments to a value — used to represent struct fields
/// (`Named<Format>`) and enum variants (`Named<VariantFormat>`).
pub struct Named<T> {
    pub name: String,
    pub doc: Doc,
    pub value: T,
}

impl<T> Named<T>
where
    T: Clone,
{
    /// Creates a new named value
    #[must_use]
    pub fn new(value: &T, name: String) -> Self {
        Self {
            name,
            doc: Doc::default(),
            value: value.clone(),
        }
    }

    /// Creates a new named value without documentation.
    #[must_use]
    pub fn without_docs(&self) -> Self {
        Self {
            doc: Doc::default(),
            ..self.clone()
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
/// A mutable cell that starts as `None` and is filled in during registry construction.
/// Used inside [`Format::Variable`] and [`VariantFormat::Variable`] to hold formats
/// that aren't known yet when the node is first created. The interior mutability
/// (`Rc<RefCell<...>>`) allows the value to be resolved after the node is already
/// embedded in a parent container.
pub struct Variable<T>(Rc<RefCell<Option<T>>>);

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
/// A single variant within a [`ContainerFormat::Enum`] — describes what data the variant carries.
///
/// For example, given `enum Msg { Ping, Text(String), Move { x: f32, y: f32 } }`,
/// `Ping` is `Unit`, `Text` is `NewType(Str)`, and `Move` is `Struct([x: F32, y: F32])`.
pub enum VariantFormat {
    /// A placeholder for a variant format not yet known at construction time.
    /// See [`Format::Variable`] for details.
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

/// Recursive traversal and resolution of format AST nodes. Provides immutable and mutable
/// visiting, variable resolution, and normalization (e.g. collapsing uniform tuples into
/// `TupleArray`).
///
/// Implemented by all AST node types: [`Format`], [`ContainerFormat`], [`VariantFormat`],
/// [`Named<T>`], and [`Variable<T>`].
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
    T: FormatHolder + fmt::Debug,
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
    pub fn borrow(&self) -> Ref<'_, Option<T>> {
        self.0.as_ref().borrow()
    }

    #[must_use]
    pub fn borrow_mut(&self) -> RefMut<'_, Option<T>> {
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
    T: FormatHolder + fmt::Debug + Clone,
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
            Self::UnitStruct(_doc) => (),
            Self::NewTypeStruct(format, _doc) => format.visit(f)?,
            Self::TupleStruct(formats, _doc) => {
                for format in formats {
                    format.visit(f)?;
                }
            }
            Self::Struct(named_formats, _doc) => {
                for format in named_formats {
                    format.visit(f)?;
                }
            }
            Self::Enum(variants, _doc) => {
                for variant in variants {
                    variant.1.visit(f)?;
                }
            }
        }
        Ok(())
    }

    fn visit_mut(&mut self, f: &mut dyn FnMut(&mut Format) -> Result<()>) -> Result<()> {
        match self {
            Self::UnitStruct(_doc) => (),
            Self::NewTypeStruct(format, _doc) => format.visit_mut(f)?,
            Self::TupleStruct(formats, _doc) => {
                for format in formats {
                    format.visit_mut(f)?;
                }
            }
            Self::Struct(named_formats, _doc) => {
                for format in named_formats {
                    format.visit_mut(f)?;
                }
            }
            Self::Enum(variants, _doc) => {
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
            | Self::Set(format)
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
            | Self::Set(format)
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

    /// Whether this format is a native/primitive type (not a container or
    /// reference).
    #[must_use]
    pub(crate) fn is_native(&self) -> bool {
        matches!(
            self,
            Format::Unit
                | Format::Bool
                | Format::I8
                | Format::I16
                | Format::I32
                | Format::I64
                | Format::I128
                | Format::U8
                | Format::U16
                | Format::U32
                | Format::U64
                | Format::U128
                | Format::F32
                | Format::F64
                | Format::Char
                | Format::Str
                | Format::Bytes
        )
    }

    /// Whether this format is a leaf type — either a native primitive or a
    /// named type reference.
    #[must_use]
    pub(crate) fn is_leaf(&self) -> bool {
        self.is_native() || matches!(self, Format::TypeName(..))
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
// `Named { name: x, value: y, doc: z }` as a map `{ x: (y, z) }`.
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
            map.serialize_entry(&self.name, &(&self.value, &self.doc))?;
            map.end()
        } else {
            let mut inner = serializer.serialize_struct("Named", 3)?;
            inner.serialize_field("name", &self.name)?;
            inner.serialize_field("value", &self.value)?;
            inner.serialize_field("doc", &self.doc)?;
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

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a single entry map")
    }

    fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: de::MapAccess<'de>,
    {
        let named_value = match access.next_entry::<String, T>()? {
            Some((name, value)) => Named {
                name,
                doc: Doc::new(),
                value,
            },
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
    doc: Doc,
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
            let NamedInternal { name, doc, value } = NamedInternal::deserialize(deserializer)?;
            Ok(Self { name, doc, value })
        }
    }
}
