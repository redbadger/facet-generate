package com.example

import com.novi.serde.JsonElementSerializer
import com.novi.serde.JsonNewTypeSerializer
import com.novi.serde.JsonPairSerializer
import com.novi.serde.JsonTripleSerializer
import com.novi.serde.JsonUnitSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.builtins.serializer

@Serializable
@SerialName("App")
data class App(
    @SerialName("entry") val entry: com.example.kv.Entry,
)

@Serializable
@SerialName("Level")
enum class Level {
    @SerialName("Low") LOW,
    @SerialName("High") HIGH;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
}

@Serializable(with = Outcome.JsonSerializer::class)
sealed interface Outcome {
    data class Score(
        val value: UInt,
    ) : Outcome

    data object Missing: Outcome

    object JsonSerializer : JsonElementSerializer<Outcome>(
        "Outcome",
        toJson = { value ->
            when (value) {
                is Score -> variant("Score", encode(UInt.serializer(), value.value))
                is Missing -> variant("Missing")
            }
        },
        fromJson = { element ->
            val (tag, content) = variant(element)
            when (tag) {
                "Score" -> Score(decode(UInt.serializer(), payload(content)))
                "Missing" -> Missing
                else -> unknownVariant(tag)
            }
        },
    )
}
