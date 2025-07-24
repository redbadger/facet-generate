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

@Serializable
data class Foo (
    val one: Boolean,
    val two: String? = null
): KeyPathMutable<Foo> {
    override fun patching(patch: PatchOperation, keyPath: List<KeyPathElement>): Foo {
        if (keyPath.isEmpty()) {
            return this.applying(patch)
        }
        return when (val field = keyPath.firstOrNull()) {
            KeyPathElement.Field("one") -> {
                val newValue = this.one.patching(patch, keyPath.drop(1))
                this.copy(one = newValue)
            }
            KeyPathElement.Field("two") -> {
                val newValue = this.two.patching(patch, keyPath.drop(1))
                this.copy(two = newValue)
            }
            else -> throw IllegalStateException("Foo does not support $field key path.")
        }
    }

    private fun applying(patch: PatchOperation): Foo {
        return when (patch) {
            is PatchOperation.Update -> patch.value.asT()
            else -> throw IllegalStateException("Foo does not support splice operations.")
        }
    }
}

