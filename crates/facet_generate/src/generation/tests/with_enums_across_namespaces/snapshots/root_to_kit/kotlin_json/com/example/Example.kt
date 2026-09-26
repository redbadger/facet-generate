package com.example

import com.novi.serde.JsonElementSerializer
import com.novi.serde.JsonNewTypeSerializer
import com.novi.serde.JsonPairSerializer
import com.novi.serde.JsonTripleSerializer
import com.novi.serde.JsonUnitSerializer
import kotlinx.serialization.EncodeDefault
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.builtins.nullable
import kotlinx.serialization.builtins.serializer

@Serializable
@SerialName("Card")
data class Card(
    @SerialName("presence") val presence: com.example.kit.Presence,
    @SerialName("shape") val shape: com.example.kit.Shape,
    @SerialName("shapes") val shapes: List<com.example.kit.Shape?>,
    @SerialName("badge") val badge: com.example.kit.Badge,
)

@Serializable
@SerialName("Presence")
data class Presence(
    @SerialName("since") val since: ULong,
)

@Serializable
@SerialName("Sighting")
data class Sighting(
    @SerialName("last_seen") val lastSeen: com.example.Presence,
)
