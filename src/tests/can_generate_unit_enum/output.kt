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
    @SerialName("Red") RED,
    @SerialName("Blue") BLUE,

    /// Green is a cool color
    @SerialName("Green") GREEN;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
}
