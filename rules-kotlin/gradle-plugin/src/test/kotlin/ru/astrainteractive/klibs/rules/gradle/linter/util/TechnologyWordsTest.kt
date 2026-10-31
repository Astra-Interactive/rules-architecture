@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.util

import kotlin.test.Test
import kotlin.test.assertEquals

class TechnologyWordsTest {

    @Test
    fun GIVEN_library_coordinates_WHEN_words_found_THEN_keeps_technology_names_and_drops_generic_words() {
        val coordinates = listOf(
            "org.mongodb:mongodb-driver-kotlin-coroutine",
            "org.jetbrains.exposed:exposed-core",
            "app.cash.sqldelight:runtime",
            "org.jetbrains.kotlinx:kotlinx-coroutines-core",
            "androidx.lifecycle:lifecycle-viewmodel",
            "io.ktor:ktor-client-okhttp",
            "androidx.compose.ui:ui"
        )

        assertEquals(
            setOf(
                "mongodb",
                "exposed",
                "sqldelight",
                "ktor"
            ),
            findTechnologyWords(coordinates)
        )
    }

    @Test
    fun GIVEN_camel_case_artifact_WHEN_words_found_THEN_keeps_its_first_word_too() {
        val coordinates = listOf(
            "com.github.MilkBowl:VaultAPI",
            "me.clip:placeholderapi"
        )

        assertEquals(
            setOf(
                "milkbowl",
                "vaultapi",
                "vault",
                "clip",
                "placeholderapi"
            ),
            findTechnologyWords(coordinates)
        )
    }
}
