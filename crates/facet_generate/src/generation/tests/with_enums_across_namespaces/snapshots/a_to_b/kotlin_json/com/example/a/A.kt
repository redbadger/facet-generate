package com.example.a

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
@SerialName("Row")
data class Row(
    val status: com.example.a.b.Status,
    val signal: com.example.a.b.Signal,
)
