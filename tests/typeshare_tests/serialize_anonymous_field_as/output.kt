package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable
@JsonClassDiscriminator("type")
sealed interface SomeEnum {
    /// The associated String contains some opaque context
    @Serializable @SerialName("Context") data class Context(val content: String) : SomeEnum

    @Serializable @SerialName("Other") data class Other(val content: Int) : SomeEnum
}
