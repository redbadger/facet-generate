package com.example

import kotlinx.serialization.*
import kotlinx.serialization.builtins.*
import kotlinx.serialization.descriptors.*
import kotlinx.serialization.encoding.*
import kotlinx.serialization.json.*
import kotlinx.serialization.modules.*

@Serializable
data class QualifiedTypes(
        val unqualified: String,
        val qualified: String,
        val qualified_vec: List<String>,
        val qualified_hashmap: Map<String, String>,
        val qualified_optional: String? = null,
        val qualfied_optional_hashmap_vec: Map<String, List<String>>? = null
)
