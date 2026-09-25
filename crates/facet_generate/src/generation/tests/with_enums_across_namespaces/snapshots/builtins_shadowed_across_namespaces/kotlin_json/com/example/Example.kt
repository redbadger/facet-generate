package com.example

import com.novi.serde.JsonElementSerializer
import com.novi.serde.JsonNewTypeSerializer
import com.novi.serde.JsonPairSerializer
import com.novi.serde.JsonTripleSerializer
import com.novi.serde.JsonUnitSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.builtins.SetSerializer
import kotlinx.serialization.builtins.serializer

@Serializable
@SerialName("Shelf")
data class Shelf(
    @SerialName("set") val set: com.example.kit.Set,
    @SerialName("ids") val ids: Set<UInt>,
    @SerialName("unit") val unit: com.example.Unit,
)

@Serializable
@SerialName("Unit")
data class Unit(
    @SerialName("value") val value: UInt,
)
