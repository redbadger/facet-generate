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
    @Serializable @SerialName("A") data object A : SomeEnum

    @Serializable @SerialName("C") data class C(val content: Int) : SomeEnum
}
