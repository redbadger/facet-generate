package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable
data class Child(
    val name: String,
)

@Serializable
sealed interface Parent {
    val serialName: String

    @Serializable
    @SerialName("Child")
    data class Child(
        val value: Child,
    ) : Parent {
        override val serialName: String = "Child"
    }
}
