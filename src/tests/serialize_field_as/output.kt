package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable
data class EditItemViewModelSaveRequest(
        val context: String,
        val values: List<EditItemSaveValue>,
        val fill_action: AutoFillItemActionRequest? = null
)
