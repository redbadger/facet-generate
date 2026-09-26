package com.example.kit

import com.novi.serde.JsonElementSerializer
import com.novi.serde.JsonNewTypeSerializer
import com.novi.serde.JsonPairSerializer
import com.novi.serde.JsonTripleSerializer
import com.novi.serde.JsonUnitSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.builtins.serializer

@Serializable
@SerialName("Badge")
data class Badge(
    @SerialName("presence") val presence: com.example.kit.Presence,
    @SerialName("shape") val shape: com.example.kit.Shape,
)

@Serializable
@SerialName("Presence")
enum class Presence {
    @SerialName("Online") ONLINE,
    @SerialName("Offline") OFFLINE;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
}

@Serializable(with = Shape.JsonSerializer::class)
sealed interface Shape {
    data class Circle(
        val value: Double,
    ) : Shape

    data object Empty: Shape

    object JsonSerializer : JsonElementSerializer<Shape>(
        "Shape",
        toJson = { value ->
            when (value) {
                is Circle -> variant("Circle", encode(Double.serializer(), value.value))
                is Empty -> variant("Empty")
            }
        },
        fromJson = { element ->
            val (tag, content) = variant(element)
            when (tag) {
                "Circle" -> Circle(decode(Double.serializer(), payload(content)))
                "Empty" -> Empty
                else -> unknownVariant(tag)
            }
        },
    )
}
