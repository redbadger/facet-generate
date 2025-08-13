package com.example

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data object Tag

@Serializable
data class Video(
    val tags: List<Tag>,
)
