package com.example

/**
 * Compatibility function for buildList, available for Kotlin versions < 1.6.0
 * This provides the same functionality as the standard library buildList function
 * introduced in Kotlin 1.6.0, ensuring compatibility with older Kotlin environments.
 */
inline fun <T> buildList(capacity: Int, builderAction: MutableList<T>.() -> Unit): List<T> {
    val list = ArrayList<T>(capacity)
    list.builderAction()
    return list
}

inline fun <T> buildList(builderAction: MutableList<T>.() -> Unit): List<T> {
    val list = mutableListOf<T>()
    list.builderAction()
    return list
}

data object CustomType

data class Types(
    val s: String,
    val staticS: String,
    val int8: Byte,
    val float: Float,
    val double: Double,
    val array: List<String>,
    val fixedLengthArray: List<String>,
    val dictionary: Map<String, Int>,
    val optionalDictionary: Map<String, Int>? = null,
    val customType: CustomType,
)
