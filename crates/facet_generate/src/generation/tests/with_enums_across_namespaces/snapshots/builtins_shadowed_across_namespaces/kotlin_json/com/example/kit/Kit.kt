package com.example.kit

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
@SerialName("Set")
data class Set(
    val value: UInt,
)

@Serializable
@SerialName("Tray")
data class Tray(
    val nothing: Unit,
    val ids: kotlin.collections.Set<UInt>,
)
