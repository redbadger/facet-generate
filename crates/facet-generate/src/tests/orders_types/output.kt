package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable data class A(val field: UInt)

@Serializable data class B(val dependsOn: A)

@Serializable data class C(val dependsOn: B)

@Serializable data class D(val dependsOn: C, val alsoDependsOn: E? = null)

@Serializable data class E(val dependsOn: D)
