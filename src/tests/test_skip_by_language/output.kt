package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable data class NotVisibleInSwift(val inner: UInt)

@Serializable data class NotVisibleInTypescript(val inner: UInt)

@Serializable
enum class EnumWithVariantsPerLanguage {
    @SerialName("NotVisibleInSwift") NOT_VISIBLE_IN_SWIFT,
    @SerialName("NotVisibleInTypescript") NOT_VISIBLE_IN_TYPESCRIPT;

    val serialName: String
        get() = javaClass.getDeclaredField(name).getAnnotation(SerialName::class.java)!!.value
}
