package com.example

import java.util.UUID

data class Foo(
    val id: UUID,
    val maybeId: UUID? = null,
)
