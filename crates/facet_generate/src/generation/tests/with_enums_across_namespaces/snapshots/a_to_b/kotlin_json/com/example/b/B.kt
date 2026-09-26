package com.example.b

import com.novi.serde.JsonElementSerializer
import com.novi.serde.JsonNewTypeSerializer
import com.novi.serde.JsonPairSerializer
import com.novi.serde.JsonTripleSerializer
import com.novi.serde.JsonUnitSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.builtins.serializer

@Serializable(with = Signal.JsonSerializer::class)
sealed interface Signal {
    data class Level(
        val value: UByte,
    ) : Signal

    data object Silent: Signal

    object JsonSerializer : JsonElementSerializer<Signal>(
        "Signal",
        toJson = { value ->
            when (value) {
                is Level -> variant("Level", encode(UByte.serializer(), value.value))
                is Silent -> variant("Silent")
            }
        },
        fromJson = { element ->
            val (tag, content) = variant(element)
            when (tag) {
                "Level" -> Level(decode(UByte.serializer(), payload(content)))
                "Silent" -> Silent
                else -> unknownVariant(tag)
            }
        },
    )
}

@Serializable
@SerialName("Status")
enum class Status {
    @SerialName("Up") UP,
    @SerialName("Down") DOWN;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
}
