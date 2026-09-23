package com.example

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
@SerialName("Shelf")
data class Shelf(
    val set: com.example.kit.Set,
    val ids: Set<UInt>,
    val unit: com.example.Unit,
)

@Serializable
@SerialName("Unit")
data class Unit(
    val value: UInt,
)
