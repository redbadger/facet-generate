package com.example

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data object Tag

@Serializable
data class Video(
    val tags: List<Tag>,
)
