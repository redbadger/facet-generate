// Copyright (c) Redbadger Ltd.
// SPDX-License-Identifier: MIT OR Apache-2.0

package com.facet.generate.msgpack

import kotlinx.serialization.InternalSerializationApi
import kotlinx.serialization.KSerializer
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.descriptors.StructureKind
import kotlinx.serialization.descriptors.buildSerialDescriptor
import kotlinx.serialization.encoding.CompositeDecoder
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder

/**
 * Serializes [Triple]<A, B, C> as a 3-element MessagePack array `[a, b, c]`,
 * matching the wire format produced by Rust's `rmp_serde::to_vec_named`
 * for a Rust 3-tuple `(A, B, C)`.
 *
 * Apply this serializer at the field level:
 *
 * ```kotlin
 * @Serializable(with = TripleAsArraySerializer::class)
 * val t: Triple<Long, ULong, Int>
 * ```
 */
@OptIn(InternalSerializationApi::class)
class TripleAsArraySerializer<A, B, C>(
    private val aSerializer: KSerializer<A>,
    private val bSerializer: KSerializer<B>,
    private val cSerializer: KSerializer<C>,
) : KSerializer<Triple<A, B, C>> {

    override val descriptor: SerialDescriptor =
        buildSerialDescriptor("kotlin.Triple", StructureKind.LIST) {
            element("first", aSerializer.descriptor)
            element("second", bSerializer.descriptor)
            element("third", cSerializer.descriptor)
        }

    override fun serialize(encoder: Encoder, value: Triple<A, B, C>) {
        val composite = encoder.beginCollection(descriptor, 3)
        composite.encodeSerializableElement(descriptor, 0, aSerializer, value.first)
        composite.encodeSerializableElement(descriptor, 1, bSerializer, value.second)
        composite.encodeSerializableElement(descriptor, 2, cSerializer, value.third)
        composite.endStructure(descriptor)
    }

    override fun deserialize(decoder: Decoder): Triple<A, B, C> {
        val composite = decoder.beginStructure(descriptor)
        val first: A
        val second: B
        val third: C
        if (composite.decodeSequentially()) {
            first = composite.decodeSerializableElement(descriptor, 0, aSerializer)
            second = composite.decodeSerializableElement(descriptor, 1, bSerializer)
            third = composite.decodeSerializableElement(descriptor, 2, cSerializer)
        } else {
            var f: A? = null
            var s: B? = null
            var t: C? = null
            loop@ while (true) {
                when (val index = composite.decodeElementIndex(descriptor)) {
                    0 -> f = composite.decodeSerializableElement(descriptor, 0, aSerializer)
                    1 -> s = composite.decodeSerializableElement(descriptor, 1, bSerializer)
                    2 -> t = composite.decodeSerializableElement(descriptor, 2, cSerializer)
                    CompositeDecoder.DECODE_DONE -> break@loop
                    else -> error("Unexpected index: $index")
                }
            }
            @Suppress("UNCHECKED_CAST")
            first = f as A
            @Suppress("UNCHECKED_CAST")
            second = s as B
            @Suppress("UNCHECKED_CAST")
            third = t as C
        }
        composite.endStructure(descriptor)
        return Triple(first, second, third)
    }
}
