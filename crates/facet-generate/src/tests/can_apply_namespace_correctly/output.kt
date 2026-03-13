package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable data class ItemDetailsFieldValue(val hello: String)

@Serializable
@JsonClassDiscriminator("t")
sealed interface AdvancedColors {
    @Serializable @SerialName("Str") data class Str(val c: String) : AdvancedColors

    @Serializable @SerialName("Number") data class Number(val c: Int) : AdvancedColors

    @Serializable
    @SerialName("NumberArray")
    data class NumberArray(val c: List<Int>) : AdvancedColors

    @Serializable
    @SerialName("ReallyCoolType")
    data class ReallyCoolType(val c: ItemDetailsFieldValue) : AdvancedColors

    @Serializable
    @SerialName("ArrayReallyCoolType")
    data class ArrayReallyCoolType(val c: List<ItemDetailsFieldValue>) : AdvancedColors

    @Serializable
    @SerialName("DictionaryReallyCoolType")
    data class DictionaryReallyCoolType(val c: Map<String, ItemDetailsFieldValue>) : AdvancedColors
}
