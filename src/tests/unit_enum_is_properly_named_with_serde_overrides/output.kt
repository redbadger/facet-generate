package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

/// This is a comment.
/// Continued lovingly here
@Serializable
enum class Colors {
    @SerialName("red")
    RED,

    @SerialName("blue")
    BLUE,

    /// Green is a cool color
    @SerialName("green-like")
    GREEN;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
}

