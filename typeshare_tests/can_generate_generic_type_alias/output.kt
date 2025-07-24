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

typealias GenericTypeAlias<T> = List<T>

typealias NonGenericAlias = GenericTypeAlias<String?>

