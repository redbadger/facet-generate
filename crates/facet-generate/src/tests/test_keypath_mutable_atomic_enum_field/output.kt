package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable data object NotDiffable

@Serializable
@JsonClassDiscriminator("type")
sealed interface InternallyTagged : KeyPathMutable<InternallyTagged> {
    @Serializable
    @SerialName("anonymousStruct")
    data class AnonymousStruct(val atomic: NotDiffable) : InternallyTagged

    override fun patching(patch: PatchOperation, keyPath: List<KeyPathElement>): InternallyTagged {
        if (keyPath.size < 2) {
            return this.applying(patch)
        }
        val variant = keyPath[0]
        val field = keyPath[1]
        return when {
            this is AnonymousStruct &&
                    variant ==
                            KeyPathElement.Variant("anonymousStruct", VariantTagType.INTERNAL) -> {
                when (field) {
                    KeyPathElement.Field("atomic") -> {
                        if (keyPath.size != 2) {
                            throw IllegalStateException(
                                    "InternallyTagged.AnonymousStruct.atomic expects an atomic update"
                            )
                        }
                        when (patch) {
                            is PatchOperation.Update -> this.copy(atomic = patch.value.asT())
                            else ->
                                    throw IllegalStateException(
                                            "InternallyTagged.AnonymousStruct.atomic is atomic and only support update patches"
                                    )
                        }
                    }
                    else ->
                            throw IllegalStateException(
                                    "InternallyTagged.AnonymousStruct does not support $field key path"
                            )
                }
            }
            else ->
                    throw IllegalStateException(
                            "InternallyTagged has no mutable $variant key path."
                    )
        }
    }

    private fun applying(patch: PatchOperation): InternallyTagged {
        when (patch) {
            is PatchOperation.Update -> return patch.value.asT()
            is PatchOperation.Splice ->
                    throw IllegalStateException(
                            "InternallyTagged does not support splice operations."
                    )
        }
    }
}
