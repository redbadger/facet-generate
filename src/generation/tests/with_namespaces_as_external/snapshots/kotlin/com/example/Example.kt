package com.example

data class Child(
    val external: com.example2.other.other.OtherParent,
)

sealed interface Parent {
    data class Child(
        val value: com.example.Child,
    ) : Parent
}
