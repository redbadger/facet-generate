package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

typealias GenericTypeAlias<T> = List<T>

typealias NonGenericAlias = GenericTypeAlias<String?>
