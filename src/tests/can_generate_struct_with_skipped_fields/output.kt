package com.example

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class MyStruct(
    val a: Int,
    val c: Int,
)
