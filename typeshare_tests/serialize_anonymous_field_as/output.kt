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

@Serializable
@JsonClassDiscriminator("type")
sealed interface SomeEnum {
    /// The associated String contains some opaque context
    @Serializable
    @SerialName("Context")
    data class Context(val content: String) : SomeEnum

    @Serializable
    @SerialName("Other")
    data class Other(val content: Int) : SomeEnum
}

