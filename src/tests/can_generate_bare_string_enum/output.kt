package com.example

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/// This is a comment.
@Serializable
enum class Colors {
    @SerialName("Red") RED,
    @SerialName("Blue") BLUE,
    @SerialName("Green") GREEN;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
}
