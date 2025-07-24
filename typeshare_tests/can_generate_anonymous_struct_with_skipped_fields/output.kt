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

@Serializable(with = SomeEnum.Serializer::class)
sealed interface SomeEnum {
    @Serializable
    data class AnonymousStruct(val all: Boolean, val except_swift: Boolean, val except_ts: Boolean) : SomeEnum

    object Serializer : KSerializer<SomeEnum> {
        override val descriptor = buildClassSerialDescriptor("SomeEnum") {
            element<AnonymousStruct>("AnonymousStruct", isOptional = true)
        }

        override fun serialize(encoder: Encoder, value: SomeEnum) {
            encoder.encodeStructure(descriptor) {
                when (value) { 
                    is AnonymousStruct -> encodeSerializableElement(
                        descriptor,
                        0,
                        AnonymousStruct.serializer(),
                        value
                    )
                }
            }
        }

        override fun deserialize(decoder: Decoder): SomeEnum {
            return decoder.decodeStructure(descriptor) {
                when (val index = decodeElementIndex(descriptor)) { 
                    0 -> {
                        return@decodeStructure decodeSerializableElement(descriptor, 0, AnonymousStruct.serializer())
                    }
                    else -> throw Exception("Unknown enum variant $index for SomeEnum")
                }
            }
        }
    }
}

