package com.example

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
@SerialName("App")
data class App(
    val entry: com.example.kv.Entry,
)

@Serializable
@SerialName("Level")
enum class Level {
    @SerialName("Low") LOW,
    @SerialName("High") HIGH;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
}

@Serializable
@SerialName("Outcome")
sealed interface Outcome {
    @Serializable
    @SerialName("Score")
    data class Score(
        val value: UInt,
    ) : Outcome

    @Serializable
    @SerialName("Missing")
    data object Missing: Outcome
}
