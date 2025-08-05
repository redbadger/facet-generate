package com.photoroom.engine

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*
import com.photoroom.engine.photogossip.interfaces.*
import com.photoroom.engine.photogossip.extensions.*
import com.photoroom.engine.misc.EngineSerialization
import com.photoroom.engine.photogossip.PatchOperation

/// This is a comment.
@Serializable
data class ArcyColors (
    val red: UByte,
    val blue: String,
    val green: List<String>
)

/// This is a comment.
@Serializable
data class MutexyColors (
    val blue: List<String>,
    val green: String
)

/// This is a comment.
@Serializable
data class RcyColors (
    val red: String,
    val blue: List<String>,
    val green: String
)

/// This is a comment.
@Serializable
data class CellyColors (
    val red: String,
    val blue: List<String>
)

/// This is a comment.
@Serializable
data class LockyColors (
    val red: String
)

/// This is a comment.
@Serializable
data class CowyColors (
    val lifetime: String
)

/// This is a comment.
@Serializable
@JsonClassDiscriminator("type")
sealed interface BoxyColors {
    @Serializable
    @SerialName("Red")
    data object Red : BoxyColors

    @Serializable
    @SerialName("Blue")
    data object Blue : BoxyColors

    @Serializable
    @SerialName("Green")
    data class Green(val content: String) : BoxyColors
}

