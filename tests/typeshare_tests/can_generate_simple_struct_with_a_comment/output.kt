package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable
data object Location

/// This is a comment.
@Serializable
data class Person (
    /// This is another comment
    val name: String,
    val age: UByte,
    val info: String? = null,
    val emails: List<String>,
    val location: Location
)

