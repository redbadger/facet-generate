package com.example

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

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
