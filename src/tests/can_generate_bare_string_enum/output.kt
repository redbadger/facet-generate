package com.example

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

/// This is a comment.
@Serializable
enum class Colors {
    @SerialName("Red") RED,
    @SerialName("Blue") BLUE,
    @SerialName("Green") GREEN;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
}
