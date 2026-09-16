package com.example

sealed interface KeywordEnum {
    data object Default: KeywordEnum

    data object Case: KeywordEnum

    data class Switch(
        val value: String,
    ) : KeywordEnum

    data class Where(
        val `in`: Int,
        val default: String,
    ) : KeywordEnum
}

/// A struct whose every field is a keyword in at least one target language.
/// Each language escapes only its own reserved words: `import` is escaped in
/// Swift and TypeScript but is a soft keyword in Kotlin, and `type` is
/// contextual everywhere, so both come through bare where they are legal.
data class KeywordFields(
    val default: String,
    val `in`: Int,
    val `class`: Boolean,
    val `object`: String,
    val static: Boolean,
    val let: String,
    val `when`: Int,
    val `is`: Boolean,
    val `fun`: String,
    val operator: String,
    val import: String,
    val type: String,
    val function: String? = null,
    /// A tuple field: the Swift plugin derives `whereField0` / `whereField1`
    /// locals from this name, which must stay unescaped.
    val where: Pair<Int, String>,
)

/// Newtype struct — its member is named `value`, a Kotlin soft keyword that
/// must not be escaped.
data class KeywordNewType(
    val value: String,
)

/// Tuple struct — its members are named `field0`, `field1`, which are never
/// keywords.
data class KeywordTuple(
    val field0: String,
    val field1: Int,
)
