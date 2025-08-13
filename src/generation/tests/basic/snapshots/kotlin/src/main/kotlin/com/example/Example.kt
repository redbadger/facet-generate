package com.example

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

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
