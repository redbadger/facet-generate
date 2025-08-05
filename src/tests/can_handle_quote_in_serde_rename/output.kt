package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

/// This is a comment.
@Serializable
enum class Colors {
    @SerialName("Green\"") GREEN;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
}
