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
 * Serializes [Pair]<A, B> as a 2-element MessagePack array `[a, b]`,
 * matching the wire format produced by Rust's `rmp_serde::to_vec_named`
 * for a Rust 2-tuple `(A, B)`.
 *
 * The default kotlinx-serialization descriptor for [Pair] treats it as a
 * class with `first` / `second` string keys, which encodes as a 2-key map
 * and does **not** match rmp-serde's array encoding.
 *
 * Apply this serializer at the field level:
 *
 * ```kotlin
 * @Serializable(with = PairAsArraySerializer::class)
 * val b: Pair<Long, ULong>
 * ```
 */
@OptIn(InternalSerializationApi::class)
class PairAsArraySerializer<A, B>(
    private val aSerializer: KSerializer<A>,
    private val bSerializer: KSerializer<B>,
) : KSerializer<Pair<A, B>> {

    override val descriptor: SerialDescriptor =
        buildSerialDescriptor("kotlin.Pair", StructureKind.LIST) {
            element("first", aSerializer.descriptor)
            element("second", bSerializer.descriptor)
        }

    override fun serialize(encoder: Encoder, value: Pair<A, B>) {
        val composite = encoder.beginCollection(descriptor, 2)
        composite.encodeSerializableElement(descriptor, 0, aSerializer, value.first)
        composite.encodeSerializableElement(descriptor, 1, bSerializer, value.second)
        composite.endStructure(descriptor)
    }

    override fun deserialize(decoder: Decoder): Pair<A, B> {
        val composite = decoder.beginStructure(descriptor)
        val first: A
        val second: B
        if (composite.decodeSequentially()) {
            first = composite.decodeSerializableElement(descriptor, 0, aSerializer)
            second = composite.decodeSerializableElement(descriptor, 1, bSerializer)
        } else {
            var f: A? = null
            var s: B? = null
            loop@ while (true) {
                when (val index = composite.decodeElementIndex(descriptor)) {
                    0 -> f = composite.decodeSerializableElement(descriptor, 0, aSerializer)
                    1 -> s = composite.decodeSerializableElement(descriptor, 1, bSerializer)
                    CompositeDecoder.DECODE_DONE -> break@loop
                    else -> error("Unexpected index: $index")
                }
            }
            @Suppress("UNCHECKED_CAST")
            first = f as A
            @Suppress("UNCHECKED_CAST")
            second = s as B
        }
        composite.endStructure(descriptor)
        return Pair(first, second)
    }
}
