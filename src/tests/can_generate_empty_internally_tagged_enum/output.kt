package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable
@JsonClassDiscriminator("type")
sealed interface Test {
    @Serializable @SerialName("AnonymousEmptyStruct") data object AnonymousEmptyStruct : Test

    @Serializable @SerialName("NoStruct") data object NoStruct : Test
}
