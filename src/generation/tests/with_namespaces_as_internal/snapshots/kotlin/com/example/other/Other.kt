package com.example.other

data class OtherChild(
    val name: String,
)

sealed interface OtherParent {
    data class Child(
        val value: com.example.other.OtherChild,
    ) : OtherParent
}
