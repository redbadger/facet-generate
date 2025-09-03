package com.example

data object Location

/// This is a comment.
data class Person(
    /// This is another comment
    val name: String,
    val age: UByte,
    val info: String? = null,
    val emails: List<String>,
    val location: Location,
)
