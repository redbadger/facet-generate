package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

/// This is a comment.
@Serializable
data class Foo(
    val a: Byte,
    val b: Short,
    val c: Int,
    val e: UByte,
    val f: UShort,
    val g: UInt,
)
