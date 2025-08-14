package com.example

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

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
