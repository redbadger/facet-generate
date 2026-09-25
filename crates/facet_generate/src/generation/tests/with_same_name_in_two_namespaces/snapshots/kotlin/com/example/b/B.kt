package com.example.b

data class Child(
    val y: UByte,
)

data class Parent(
    val first: com.example.a.Child,
    val second: com.example.b.Child,
)
