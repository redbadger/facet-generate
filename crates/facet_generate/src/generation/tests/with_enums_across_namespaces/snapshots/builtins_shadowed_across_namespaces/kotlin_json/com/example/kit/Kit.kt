package com.example.kit

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
@SerialName("Set")
data class Set(
    @SerialName("value") val value: UInt,
)

@Serializable(with = Tray.JsonSerializer::class)
data class Tray(
    val nothing: Unit,
    val ids: kotlin.collections.Set<UInt>,
) {
    object JsonSerializer : JsonElementSerializer<Tray>(
        "Tray",
        toJson = { value ->
            obj(
                "nothing" to encode(JsonUnitSerializer, value.nothing),
                "ids" to encode(SetSerializer(UInt.serializer()), value.ids),
            )
        },
        fromJson = { element ->
            val fields = fields(element)
            Tray(
                nothing = decode(JsonUnitSerializer, fields.required("nothing")),
                ids = decode(SetSerializer(UInt.serializer()), fields.required("ids")),
            )
        },
    )
}
