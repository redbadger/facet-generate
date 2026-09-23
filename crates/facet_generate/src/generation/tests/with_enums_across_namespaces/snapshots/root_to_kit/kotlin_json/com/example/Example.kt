package com.example

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
@SerialName("Card")
data class Card(
    val presence: com.example.kit.Presence,
    val shape: com.example.kit.Shape,
    val shapes: List<com.example.kit.Shape?>,
    val badge: com.example.kit.Badge,
)

@Serializable
@SerialName("Presence")
data class Presence(
    val since: ULong,
)

@Serializable
@SerialName("Sighting")
data class Sighting(
    val lastSeen: com.example.Presence,
)
