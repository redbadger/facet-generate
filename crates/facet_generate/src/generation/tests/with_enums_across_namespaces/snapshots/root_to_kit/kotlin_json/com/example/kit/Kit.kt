package com.example.kit

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
@SerialName("Badge")
data class Badge(
    val presence: com.example.kit.Presence,
    val shape: com.example.kit.Shape,
)

@Serializable
@SerialName("Presence")
enum class Presence {
    @SerialName("Online") ONLINE,
    @SerialName("Offline") OFFLINE;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
}

@Serializable
@SerialName("Shape")
sealed interface Shape {
    @Serializable
    @SerialName("Circle")
    data class Circle(
        val value: Double,
    ) : Shape

    @Serializable
    @SerialName("Empty")
    data object Empty: Shape
}
