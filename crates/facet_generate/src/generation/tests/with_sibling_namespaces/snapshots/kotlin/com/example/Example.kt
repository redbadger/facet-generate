package com.example

sealed interface Root {
    data class Feature(
        val value: com.example.feature.FeatureView,
    ) : Root
}
