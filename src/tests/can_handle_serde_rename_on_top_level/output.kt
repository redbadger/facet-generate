package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable data object OtherType

/// This is a comment.
@Serializable
data class PersonTwo(
        val name: String,
        val age: UByte,
        val extraSpecialFieldOne: Int,
        val extraSpecialFieldTwo: List<String>? = null,
        val nonStandardDataType: OtherType,
        val nonStandardDataTypeInArray: List<OtherType>? = null
)
