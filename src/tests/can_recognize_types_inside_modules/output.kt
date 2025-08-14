package com.example

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class A(
    val field: UInt,
)

@Serializable
data class AB(
    val field: UInt,
)

@Serializable
data class ABC(
    val field: UInt,
)

@Serializable
data class OutsideOfModules(
    val field: UInt,
)
