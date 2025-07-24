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
enum class Colors {
    @SerialName("Red")
    RED,

    @SerialName("Blue")
    BLUE,

    @SerialName("Green")
    GREEN;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
}

