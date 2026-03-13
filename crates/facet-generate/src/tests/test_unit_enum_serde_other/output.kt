package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

/// This is a comment.
@Serializable(with = Source.Serializer::class)
enum class Source {
    @SerialName("Embedded") EMBEDDED,
    @SerialName("GoogleFont") GOOGLE_FONT,
    @SerialName("Custom") CUSTOM,
    @SerialName("Unknown") UNKNOWN;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value

    object Serializer : KSerializer<Source> {
        override val descriptor: SerialDescriptor =
                PrimitiveSerialDescriptor("Source", PrimitiveKind.STRING)

        override fun serialize(encoder: Encoder, value: Source) {
            encoder.encodeString(value.serialName)
        }

        override fun deserialize(decoder: Decoder): Source {
            return decoder.decodeString().let { value ->
                Source.entries.firstOrNull { it.serialName == value } ?: UNKNOWN
            }
        }
    }
}
