package com.example.kv

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
@SerialName("Entry")
data class Entry(
    val level: com.example.Level,
    val outcome: com.example.Outcome,
)
