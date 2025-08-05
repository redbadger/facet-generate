package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

/// This is a comment.
@Serializable
data class Things(
        @SerialName("bla") val bla: String,
        @SerialName("label") val label: String? = null,
        @SerialName("label-left") val label_left: String? = null
)
