package com.example.b

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
@SerialName("Signal")
sealed interface Signal {
    @Serializable
    @SerialName("Level")
    data class Level(
        val value: UByte,
    ) : Signal

    @Serializable
    @SerialName("Silent")
    data object Silent: Signal
}

@Serializable
@SerialName("Status")
enum class Status {
    @SerialName("Up") UP,
    @SerialName("Down") DOWN;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
}
