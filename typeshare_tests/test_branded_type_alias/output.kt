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

typealias SimpleAlias1 = String

typealias SimpleAlias2 = String

@Serializable(with = BrandedStringAlias.Serializer::class)
data class BrandedStringAlias (
    val value: String
) {
    object Serializer : KSerializer<BrandedStringAlias> {
        override val descriptor: SerialDescriptor = String.serializer().descriptor

        override fun serialize(encoder: Encoder, value: BrandedStringAlias) {
            encoder.encodeSerializableValue(String.serializer(), value.value)
        }

        override fun deserialize(decoder: Decoder): BrandedStringAlias {
            return BrandedStringAlias(decoder.decodeSerializableValue(String.serializer()))
        }
    }
}

@Serializable(with = BrandedOptionalStringAlias.Serializer::class)
data class BrandedOptionalStringAlias (
    val value: String?
) {
    object Serializer : KSerializer<BrandedOptionalStringAlias> {
        override val descriptor: SerialDescriptor = String.serializer().descriptor

        override fun serialize(encoder: Encoder, value: BrandedOptionalStringAlias) {
            encoder.encodeNullableSerializableValue(String.serializer(), value.value)
        }

        override fun deserialize(decoder: Decoder): BrandedOptionalStringAlias {
            return BrandedOptionalStringAlias(decoder.decodeNullableSerializableValue(String.serializer()))
        }
    }
}

@Serializable(with = BrandedU32Alias.Serializer::class)
data class BrandedU32Alias (
    val value: UInt
) {
    object Serializer : KSerializer<BrandedU32Alias> {
        override val descriptor: SerialDescriptor = UInt.serializer().descriptor

        override fun serialize(encoder: Encoder, value: BrandedU32Alias) {
            encoder.encodeSerializableValue(UInt.serializer(), value.value)
        }

        override fun deserialize(decoder: Decoder): BrandedU32Alias {
            return BrandedU32Alias(decoder.decodeSerializableValue(UInt.serializer()))
        }
    }
}

@Serializable
data class MyStruct (
    val field: UInt,
    val other_field: String
)

@Serializable(with = BrandedStructAlias.Serializer::class)
data class BrandedStructAlias (
    val value: MyStruct
) {
    object Serializer : KSerializer<BrandedStructAlias> {
        override val descriptor: SerialDescriptor = MyStruct.serializer().descriptor

        override fun serialize(encoder: Encoder, value: BrandedStructAlias) {
            encoder.encodeSerializableValue(MyStruct.serializer(), value.value)
        }

        override fun deserialize(decoder: Decoder): BrandedStructAlias {
            return BrandedStructAlias(decoder.decodeSerializableValue(MyStruct.serializer()))
        }
    }
}

