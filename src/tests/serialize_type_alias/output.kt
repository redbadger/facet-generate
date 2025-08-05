package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

typealias AlsoString = String

typealias Uuid = String

/// Unique identifier for an Account
typealias AccountUuid = Uuid

typealias ItemUuid = String
