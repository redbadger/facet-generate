package com.example

sealed interface BoolResult {
    data class Ok(
        val value: Boolean,
    ) : BoolResult

    data class Err(
        val value: String,
    ) : BoolResult
}

data class Delete(
    val key: String,
)

data class Exists(
    val key: String,
)

data class Get(
    val key: String,
)

data class Keys(
    val items: List<String>,
    val nextCursor: ULong,
)

sealed interface KeysResult {
    data class Ok(
        val value: com.example.Keys,
    ) : KeysResult

    data class Err(
        val value: String,
    ) : KeysResult
}

data class ListKeys(
    val prefix: String,
    val cursor: ULong,
)

data class Set(
    val key: String,
    val value: Bytes,
)

data class Store(
    val tags: kotlin.collections.Set<String>,
    val entries: Map<String, String>,
    val blob: List<UByte>,
    val pair: Pair<Int, String>,
)

sealed interface ValueResult {
    data class Ok(
        val value: List<UByte>? = null,
    ) : ValueResult

    data class Err(
        val value: String,
    ) : ValueResult
}
