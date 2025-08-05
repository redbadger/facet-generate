package com.photoroom.engine

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*
import com.photoroom.engine.photogossip.interfaces.*
import com.photoroom.engine.photogossip.extensions.*
import com.photoroom.engine.misc.EngineSerialization
import com.photoroom.engine.photogossip.PatchOperation

@Serializable
data class A (
    val field: UInt
)

@Serializable
data class B (
    val dependsOn: A
)

@Serializable
data class C (
    val dependsOn: B
)

@Serializable
data class D (
    val dependsOn: C,
    val alsoDependsOn: E? = null
)

@Serializable
data class E (
    val dependsOn: D
)

