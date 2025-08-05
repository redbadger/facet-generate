package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable
data class Bar(val one: String) : KeyPathMutable<Bar> {
    override fun patching(patch: PatchOperation, keyPath: List<KeyPathElement>): Bar {
        if (keyPath.isEmpty()) {
            return this.applying(patch)
        }
        return when (val field = keyPath.firstOrNull()) {
            KeyPathElement.Field("one") -> {
                val newValue = this.one.patching(patch, keyPath.drop(1))
                this.copy(one = newValue)
            }
            else -> throw IllegalStateException("Bar does not support $field key path.")
        }
    }

    private fun applying(patch: PatchOperation): Bar {
        return when (patch) {
            is PatchOperation.Update -> patch.value.asT()
            else -> throw IllegalStateException("Bar does not support splice operations.")
        }
    }
}

@Serializable
data class Foo(val bar: Bar? = null) : KeyPathMutable<Foo> {
    override fun patching(patch: PatchOperation, keyPath: List<KeyPathElement>): Foo {
        if (keyPath.isEmpty()) {
            return this.applying(patch)
        }
        return when (val field = keyPath.firstOrNull()) {
            KeyPathElement.Field("bar") -> {
                val newValue = this.bar.patching(patch, keyPath.drop(1))
                this.copy(bar = newValue)
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
