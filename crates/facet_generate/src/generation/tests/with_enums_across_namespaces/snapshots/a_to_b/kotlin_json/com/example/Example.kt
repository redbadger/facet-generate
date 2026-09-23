package com.example

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
@SerialName("App")
data class App(
    val row: com.example.a.Row,
)
