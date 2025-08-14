package com.example

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data object Location

/// This is a comment.
@Serializable
data class Person(
    /// This is another comment
    val name: String,
    val age: UByte,
    val info: String? = null,
    val emails: List<String>,
    val location: Location,
)
