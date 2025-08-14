package com.example

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data object CustomType

@Serializable
data class Types(
    val s: String,
    val static_s: String,
    val int8: Byte,
    val float: Float,
    val double: Double,
    val array: List<String>,
    val fixed_length_array: List<String>,
    val dictionary: Map<String, Int>,
    val optional_dictionary: Map<String, Int>? = null,
    val custom_type: CustomType,
)
