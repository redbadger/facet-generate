package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

typealias OptionalU32 = UInt?

typealias OptionalU16 = UShort?

@Serializable data class FooBar(val foo: OptionalU32, val bar: OptionalU16)
