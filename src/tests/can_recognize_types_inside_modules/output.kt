package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable
data class A (
    val field: UInt
)

@Serializable
data class AB (
    val field: UInt
)

@Serializable
data class ABC (
    val field: UInt
)

@Serializable
data class OutsideOfModules (
    val field: UInt
)

