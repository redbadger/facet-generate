package com.example

data class Child(
    val name: String,
)

sealed interface Parent {
    data class Child(
        val value: com.example.Child,
    ) : Parent
}
