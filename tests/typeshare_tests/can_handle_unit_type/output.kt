package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

/// This struct has a unit field
@Serializable data class StructHasVoidType(val thisIsAUnit: Unit)

/// This enum has a variant associated with unit data
@Serializable
@JsonClassDiscriminator("type")
sealed interface EnumHasVoidType {
    @Serializable @SerialName("hasAUnit") data class HasAUnit(val content: Unit) : EnumHasVoidType
}
