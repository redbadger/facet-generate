//! Swift reserved words.
//!
//! Source: *The Swift Programming Language* — Language Reference → Lexical
//! Structure → Keywords and Punctuation. Keywords used in declarations, in
//! statements, and in expressions and types.
//!
//! `#`-prefixed keywords cannot collide with a generated identifier, and
//! contextual keywords (`get`, `set`, `Type`, `Protocol`, …) are legal
//! identifiers, so neither group is listed.

use std::borrow::Cow;

use heck::ToLowerCamelCase;

use crate::{
    generation::{
        config::CodeGeneratorConfig,
        naming::{EscapeStyle, ForbiddenNames, NamingRules, qualify},
    },
    reflection::format::Namespace,
};

/// Swift keywords, sorted.
pub(crate) const KEYWORDS: &[&str] = &[
    "Any",
    "Self",
    "as",
    "associatedtype",
    "await",
    "borrowing",
    "break",
    "case",
    "catch",
    "class",
    "consuming",
    "continue",
    "default",
    "defer",
    "deinit",
    "do",
    "else",
    "enum",
    "extension",
    "fallthrough",
    "false",
    "fileprivate",
    "for",
    "func",
    "guard",
    "if",
    "import",
    "in",
    "init",
    "inout",
    "internal",
    "is",
    "let",
    "nil",
    "nonisolated",
    "open",
    "operator",
    "precedencegroup",
    "private",
    "protocol",
    "public",
    "repeat",
    "rethrows",
    "return",
    "self",
    "static",
    "struct",
    "subscript",
    "super",
    "switch",
    "throw",
    "throws",
    "true",
    "try",
    "typealias",
    "var",
    "where",
    "while",
];

/// Standard-library types the emitter writes bare, and the fully qualified
/// form to use instead when the module declares a type of the same name.
/// Sorted by the bare name.
///
/// A declaration in the generated module outranks the implicit `Swift` module,
/// so `Set<String>` in a module that also declares `struct Set` would resolve
/// to the struct.
pub(crate) const QUALIFIED: &[(&str, &str)] = &[
    ("Bool", "Swift.Bool"),
    ("Character", "Swift.Character"),
    ("Codable", "Swift.Codable"),
    ("CodingKey", "Swift.CodingKey"),
    ("Decoder", "Swift.Decoder"),
    ("DecodingError", "Swift.DecodingError"),
    ("Double", "Swift.Double"),
    ("Encoder", "Swift.Encoder"),
    ("EncodingError", "Swift.EncodingError"),
    ("Equatable", "Swift.Equatable"),
    ("Float", "Swift.Float"),
    ("Hashable", "Swift.Hashable"),
    ("Int16", "Swift.Int16"),
    ("Int32", "Swift.Int32"),
    ("Int64", "Swift.Int64"),
    ("Int8", "Swift.Int8"),
    ("Set", "Swift.Set"),
    ("String", "Swift.String"),
    ("UInt16", "Swift.UInt16"),
    ("UInt32", "Swift.UInt32"),
    ("UInt64", "Swift.UInt64"),
    ("UInt8", "Swift.UInt8"),
    ("UUID", "Foundation.UUID"),
    ("Void", "Swift.Void"),
];

/// The public top-level types of the `Swift` and `_Concurrency` modules
/// (structs, enums, classes, protocols, actors and typealiases), less those
/// that start with `_` and those no namespace becomes (`SIMD2`, `UTF8`).
/// Sorted.
///
/// Swift looks a qualifier up as a type before it looks for a module, and
/// every module imports both, so a module named like one of these cannot be
/// qualified, whether or not the generated code writes the type:
/// `String.Foo` looks for `Foo` in `Swift.String`.
//
// Generated from Apple Swift 6.4 (swiftlang-6.4.0.34.1), macOS 27.0 SDK:
// every line of `$(xcrun --show-sdk-path)/usr/lib/swift/{Swift,_Concurrency}
// .swiftmodule/arm64e-apple-macos.swiftinterface` that starts in column 0
// and matches `\b(public|open)\b( [a-z]+)*? (struct|enum|class|protocol|
// actor|typealias) (\w+)`, keeping each name that does not start with `_`
// and for which `name.to_snake_case().to_upper_camel_case() == name` (heck
// 0.5), sorted bytewise.
pub(crate) const STDLIB_TYPES: &[&str] = &[
    "Actor",
    "AdditiveArithmetic",
    "AnyActor",
    "AnyBidirectionalCollection",
    "AnyClass",
    "AnyCollection",
    "AnyHashable",
    "AnyIndex",
    "AnyIterator",
    "AnyKeyPath",
    "AnyObject",
    "AnyRandomAccessCollection",
    "AnySequence",
    "Array",
    "ArrayLiteralConvertible",
    "ArraySlice",
    "AsyncCompactMapSequence",
    "AsyncDropFirstSequence",
    "AsyncDropWhileSequence",
    "AsyncFilterSequence",
    "AsyncFlatMapSequence",
    "AsyncIteratorProtocol",
    "AsyncMapSequence",
    "AsyncPrefixSequence",
    "AsyncPrefixWhileSequence",
    "AsyncSequence",
    "AsyncStream",
    "AsyncThrowingCompactMapSequence",
    "AsyncThrowingDropWhileSequence",
    "AsyncThrowingFilterSequence",
    "AsyncThrowingFlatMapSequence",
    "AsyncThrowingMapSequence",
    "AsyncThrowingPrefixWhileSequence",
    "AsyncThrowingStream",
    "AutoreleasingUnsafeMutablePointer",
    "BidirectionalCollection",
    "BidirectionalIndexable",
    "BidirectionalSlice",
    "BinaryFloatingPoint",
    "BinaryInteger",
    "BitwiseCopyable",
    "Bool",
    "BooleanLiteralConvertible",
    "BooleanLiteralType",
    "BorrowingIteratorAdapter",
    "BorrowingIteratorProtocol",
    "ByteOrder",
    "CBool",
    "CChar",
    "CChar16",
    "CChar32",
    "CChar8",
    "CDouble",
    "CFloat",
    "CFloat16",
    "CInt",
    "CLong",
    "CLongDouble",
    "CLongLong",
    "CShort",
    "CSignedChar",
    "CUnsignedChar",
    "CUnsignedInt",
    "CUnsignedLong",
    "CUnsignedLongLong",
    "CUnsignedShort",
    "CVaListPointer",
    "CVarArg",
    "CWideChar",
    "CancellationError",
    "CaseIterable",
    "Character",
    "CheckedContinuation",
    "Clock",
    "ClosedRange",
    "ClosedRangeIndex",
    "Codable",
    "CodingKey",
    "CodingKeyRepresentable",
    "CodingUserInfoKey",
    "Collection",
    "CollectionDifference",
    "CollectionOfOne",
    "CommandLine",
    "Comparable",
    "ConcurrentValue",
    "ContiguousArray",
    "Continuation",
    "ContinuousClock",
    "ConvertibleFromBytes",
    "ConvertibleToBytes",
    "Copyable",
    "CountableClosedRange",
    "CountablePartialRangeFrom",
    "CountableRange",
    "CustomDebugStringConvertible",
    "CustomLeafReflectable",
    "CustomPlaygroundDisplayConvertible",
    "CustomPlaygroundQuickLookable",
    "CustomReflectable",
    "CustomStringConvertible",
    "Decodable",
    "Decoder",
    "DecodingError",
    "DefaultBidirectionalIndices",
    "DefaultIndices",
    "DefaultRandomAccessIndices",
    "DefaultStringInterpolation",
    "Dictionary",
    "DictionaryIndex",
    "DictionaryIterator",
    "DictionaryLiteral",
    "DictionaryLiteralConvertible",
    "DiscardingTaskGroup",
    "DiscontiguousSlice",
    "Double",
    "DropFirstSequence",
    "DropWhileSequence",
    "Duration",
    "DurationProtocol",
    "EmptyCollection",
    "EmptyIterator",
    "Encodable",
    "Encoder",
    "EncodingError",
    "EnumeratedIterator",
    "EnumeratedSequence",
    "Equatable",
    "Error",
    "Escapable",
    "Executor",
    "ExecutorJob",
    "ExpressibleByArrayLiteral",
    "ExpressibleByBooleanLiteral",
    "ExpressibleByDictionaryLiteral",
    "ExpressibleByExtendedGraphemeClusterLiteral",
    "ExpressibleByFloatLiteral",
    "ExpressibleByIntegerLiteral",
    "ExpressibleByNilLiteral",
    "ExpressibleByStringInterpolation",
    "ExpressibleByStringLiteral",
    "ExpressibleByUnicodeScalarLiteral",
    "ExtendedGraphemeClusterLiteralConvertible",
    "ExtendedGraphemeClusterType",
    "FixedWidthInteger",
    "FlattenBidirectionalCollection",
    "FlattenBidirectionalCollectionIndex",
    "FlattenCollection",
    "FlattenCollectionIndex",
    "FlattenSequence",
    "Float",
    "Float16",
    "Float32",
    "Float64",
    "Float80",
    "FloatLiteralConvertible",
    "FloatLiteralType",
    "FloatingPoint",
    "FloatingPointClassification",
    "FloatingPointRoundingRule",
    "FloatingPointSign",
    "FullyInhabited",
    "GlobalActor",
    "Hashable",
    "Hasher",
    "Identifiable",
    "ImplicitlyUnwrappedOptional",
    "Indexable",
    "IndexableBase",
    "IndexingIterator",
    "InlineArray",
    "InstantProtocol",
    "Int",
    "Int128",
    "Int16",
    "Int32",
    "Int64",
    "Int8",
    "IntegerLiteralConvertible",
    "IntegerLiteralType",
    "Iterable",
    "IteratorOverOne",
    "IteratorProtocol",
    "IteratorSequence",
    "Job",
    "JobPriority",
    "JoinedIterator",
    "JoinedSequence",
    "KeyPath",
    "KeyValuePairs",
    "KeyedDecodingContainer",
    "KeyedDecodingContainerProtocol",
    "KeyedEncodingContainer",
    "KeyedEncodingContainerProtocol",
    "LazyBidirectionalCollection",
    "LazyCollection",
    "LazyCollectionProtocol",
    "LazyDropWhileBidirectionalCollection",
    "LazyDropWhileCollection",
    "LazyDropWhileIndex",
    "LazyDropWhileIterator",
    "LazyDropWhileSequence",
    "LazyFilterBidirectionalCollection",
    "LazyFilterCollection",
    "LazyFilterIndex",
    "LazyFilterIterator",
    "LazyFilterSequence",
    "LazyMapBidirectionalCollection",
    "LazyMapCollection",
    "LazyMapIterator",
    "LazyMapRandomAccessCollection",
    "LazyMapSequence",
    "LazyPrefixWhileBidirectionalCollection",
    "LazyPrefixWhileCollection",
    "LazyPrefixWhileIndex",
    "LazyPrefixWhileIterator",
    "LazyPrefixWhileSequence",
    "LazyRandomAccessCollection",
    "LazySequence",
    "LazySequenceProtocol",
    "LosslessStringConvertible",
    "MainActor",
    "ManagedBuffer",
    "ManagedBufferPointer",
    "MemoryLayout",
    "Mirror",
    "MirrorPath",
    "MutableBidirectionalSlice",
    "MutableCollection",
    "MutableIndexable",
    "MutableRandomAccessSlice",
    "MutableRangeReplaceableBidirectionalSlice",
    "MutableRangeReplaceableRandomAccessSlice",
    "MutableRangeReplaceableSlice",
    "MutableRawSpan",
    "MutableRef",
    "MutableSlice",
    "MutableSpan",
    "Never",
    "NilLiteralConvertible",
    "Numeric",
    "ObjectIdentifier",
    "OpaquePointer",
    "OptionSet",
    "Optional",
    "OutputRawSpan",
    "OutputSpan",
    "PartialAsyncTask",
    "PartialKeyPath",
    "PartialRangeFrom",
    "PartialRangeThrough",
    "PartialRangeUpTo",
    "PlaygroundQuickLook",
    "PrefixSequence",
    "RandomAccessCollection",
    "RandomAccessIndexable",
    "RandomAccessSlice",
    "RandomNumberGenerator",
    "Range",
    "RangeExpression",
    "RangeReplaceableBidirectionalSlice",
    "RangeReplaceableCollection",
    "RangeReplaceableIndexable",
    "RangeReplaceableRandomAccessSlice",
    "RangeReplaceableSlice",
    "RangeSet",
    "RawRepresentable",
    "RawSpan",
    "Ref",
    "ReferenceWritableKeyPath",
    "Repeated",
    "Result",
    "ReversedCollection",
    "ReversedIndex",
    "ReversedRandomAccessCollection",
    "Sendable",
    "SendableMetatype",
    "Sequence",
    "SerialExecutor",
    "Set",
    "SetAlgebra",
    "SetIndex",
    "SetIterator",
    "SignedInteger",
    "SignedNumeric",
    "SingleValueDecodingContainer",
    "SingleValueEncodingContainer",
    "Slice",
    "Span",
    "StaticBigInt",
    "StaticString",
    "StrideThrough",
    "StrideThroughIterator",
    "StrideTo",
    "StrideToIterator",
    "Strideable",
    "String",
    "StringInterpolationConvertible",
    "StringInterpolationProtocol",
    "StringLiteralConvertible",
    "StringLiteralType",
    "StringProtocol",
    "Substring",
    "SuspendingClock",
    "SystemRandomNumberGenerator",
    "Task",
    "TaskExecutor",
    "TaskGroup",
    "TaskLocal",
    "TaskPriority",
    "TextOutputStream",
    "TextOutputStreamable",
    "ThrowingDiscardingTaskGroup",
    "ThrowingTaskGroup",
    "UInt",
    "UInt128",
    "UInt16",
    "UInt32",
    "UInt64",
    "UInt8",
    "UnboundedRange",
    "UnfoldFirstSequence",
    "UnfoldSequence",
    "Unicode",
    "UnicodeCodec",
    "UnicodeDecodingResult",
    "UnicodeScalar",
    "UnicodeScalarLiteralConvertible",
    "UnicodeScalarType",
    "UniqueArray",
    "UniqueBox",
    "UnkeyedDecodingContainer",
    "UnkeyedEncodingContainer",
    "Unmanaged",
    "UnownedJob",
    "UnownedSerialExecutor",
    "UnownedTaskExecutor",
    "UnsafeBufferPointer",
    "UnsafeBufferPointerIterator",
    "UnsafeConcurrentValue",
    "UnsafeContinuation",
    "UnsafeCurrentTask",
    "UnsafeMutableBufferPointer",
    "UnsafeMutablePointer",
    "UnsafeMutableRawBufferPointer",
    "UnsafeMutableRawBufferPointerIterator",
    "UnsafeMutableRawPointer",
    "UnsafePointer",
    "UnsafeRawBufferPointer",
    "UnsafeRawBufferPointerIterator",
    "UnsafeRawPointer",
    "UnsafeSendable",
    "UnsafeThrowingContinuation",
    "UnsignedInteger",
    "Void",
    "WritableKeyPath",
    "Zip2Iterator",
    "Zip2Sequence",
];

/// SDK modules that `Foundation` loads, with the clause that says how the
/// generated code reaches each. Sorted by name.
///
/// A target of the same name would stand in for the SDK's module, which
/// then depends on the target: "module dependency cycle".
pub(crate) const FOUNDATION_MODULES: &[(&str, &str)] = &[
    ("Combine", "which `Foundation` imports"),
    ("CoreFoundation", "which `Foundation` imports"),
    ("Darwin", "which `Foundation` imports"),
    ("Dispatch", "which `Foundation` imports"),
    ("Foundation", "which the generated code imports"),
    ("ObjectiveC", "which `Foundation` imports"),
    ("Observation", "which `Foundation` imports"),
    ("System", "which `Foundation` imports"),
];

/// Type names the generated module already uses for something else, with the
/// clause that names what each collides with. Sorted by name.
pub(crate) const FORBIDDEN_TYPES: ForbiddenNames = &[
    ("Any", "the Swift type `Any`"),
    (
        "BinaryDeserializer",
        "the runtime type `Serde.BinaryDeserializer`",
    ),
    (
        "BinarySerializer",
        "the runtime type `Serde.BinarySerializer`",
    ),
    (
        "BincodeDeserializer",
        "the runtime type `Serde.BincodeDeserializer`",
    ),
    (
        "BincodeSerializer",
        "the runtime type `Serde.BincodeSerializer`",
    ),
    (
        "DeserializationError",
        "the runtime type `Serde.DeserializationError`",
    ),
    ("Deserializer", "the runtime protocol `Serde.Deserializer`"),
    ("Foundation", "the `Foundation` module"),
    ("Indirect", "the generated `@Indirect` property wrapper"),
    ("Int128", "the Swift type `Int128`"),
    ("Protocol", "the Swift metatype keyword `Protocol`"),
    ("Self", "the Swift implicit type reference `Self`"),
    ("Serde", "the `Serde` module"),
    (
        "SerializationError",
        "the runtime type `Serde.SerializationError`",
    ),
    ("Serializer", "the runtime protocol `Serde.Serializer`"),
    ("Type", "the Swift metatype keyword `Type`"),
    ("UInt128", "the Swift type `UInt128`"),
];

/// Property names the generated code cannot accommodate, with the clause
/// explaining why. Sorted by name.
pub(crate) const FORBIDDEN_MEMBERS: ForbiddenNames = &[
    (
        "Self",
        "is the implicit type reference inside every Swift type",
    ),
    (
        "deserializer",
        "shadows the `deserializer` parameter in the generated deserialize method",
    ),
    ("hashValue", "Swift synthesises for every `Hashable` type"),
    (
        "index",
        "shadows the `index` local in the generated deserialize method",
    ),
    ("init", "is the initializer keyword in every Swift type"),
    ("self", "is the implicit receiver inside every Swift type"),
    (
        "serializer",
        "shadows the `serializer` parameter in the generated serialize method",
    ),
];

/// Swift writes container and case-carrying variant names verbatim.
fn type_case(name: &str) -> String {
    name.to_string()
}

fn member_case(name: &str) -> String {
    name.to_lower_camel_case()
}

/// The naming rules for this language.
pub(crate) const RULES: NamingRules = NamingRules {
    language: "Swift",
    reserved_words: KEYWORDS,
    escape_style: EscapeStyle::Backticks,
    forbidden_types: FORBIDDEN_TYPES,
    forbidden_members: FORBIDDEN_MEMBERS,
    format_bound_types: &[],
    type_case,
    member_case,
    variants_are_types: false,
    member_equals_type_forbidden: false,
    numbered_components_forbidden: false,
};

/// Returns `true` if a type spelled `name` is in scope in the module: one it
/// declares, or one declared by a module it imports (a namespaced module
/// imports the root package's when it references a ROOT type).
pub(crate) fn shadows(name: &str, config: &CodeGeneratorConfig) -> bool {
    let imports_root = config.root_package() != config.module_name()
        && config
            .external_definitions
            .contains_key(config.root_package());
    config.declared_type_names.contains(name)
        || config.registry_type_names.iter().any(|declared| {
            declared.name == name
                && match &declared.namespace {
                    Namespace::Named(namespace) => {
                        config.external_definitions.contains_key(namespace)
                    }
                    Namespace::Root => imports_root,
                }
        })
}

/// The Swift spelling of the builtin type `name`: fully qualified when a
/// declaration in the generated module shadows it, and bare otherwise.
pub(crate) fn builtin<'a>(name: &'a str, config: &CodeGeneratorConfig) -> Cow<'a, str> {
    qualify(name, QUALIFIED, |n| shadows(n, config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_tables_are_sorted_for_binary_search() {
        assert!(
            STDLIB_TYPES.windows(2).all(|w| w[0] < w[1]),
            "STDLIB_TYPES must be sorted"
        );
        assert!(
            FOUNDATION_MODULES.windows(2).all(|w| w[0].0 < w[1].0),
            "FOUNDATION_MODULES must be sorted by name"
        );
        assert!(
            QUALIFIED.windows(2).all(|w| w[0].0 < w[1].0),
            "QUALIFIED must be sorted by the bare name"
        );
        assert!(
            FORBIDDEN_TYPES.windows(2).all(|w| w[0].0 < w[1].0),
            "FORBIDDEN_TYPES must be sorted by name"
        );
        assert!(
            FORBIDDEN_MEMBERS.windows(2).all(|w| w[0].0 < w[1].0),
            "FORBIDDEN_MEMBERS must be sorted by name"
        );
    }

    #[test]
    fn keywords_are_sorted_and_unique() {
        assert!(
            KEYWORDS.windows(2).all(|w| w[0] < w[1]),
            "KEYWORDS must be sorted"
        );
    }

    #[test]
    fn escapes_keywords_with_backticks() {
        assert_eq!(RULES.escape("default"), "`default`");
        assert_eq!(RULES.escape("case"), "`case`");
        assert_eq!(RULES.escape("in"), "`in`");
        assert_eq!(RULES.escape("Self"), "`Self`");
    }

    #[test]
    fn leaves_ordinary_and_contextual_words_alone() {
        assert_eq!(RULES.escape("value"), "value");
        assert_eq!(RULES.escape("field0"), "field0");
        assert_eq!(RULES.escape("get"), "get");
        assert_eq!(RULES.escape("set"), "set");
        assert_eq!(RULES.escape("type"), "type");
    }

    #[test]
    fn escaping_is_idempotent() {
        assert_eq!(RULES.escape("`default`"), "`default`");
    }
}
