@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.compose

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ComposeConstantCaseTest {
    private val rule = ComposeConstantCase(Config.empty)

    @Test
    fun GIVEN_composable_file_with_miscased_values_WHEN_linted_THEN_reports_each() {
        val code = """
            private const val previewRowCount = 20
            private val card_top_spacing = 64
            private val CARD_SPACING = 64
            private const val PREVIEW_ROW_COUNT = 20
            private val PreviewCanvasColor = 0xFF1F1F1F
            private var counter = 0

            @Composable
            fun Card() = Unit
        """.trimIndent()

        val messages = rule.lint(code).map { finding -> finding.message }

        assertEquals(
            listOf(
                "Name the constant 'previewRowCount' in UPPER_SNAKE case: a const val of a composable or theme " +
                    "file is UPPER_SNAKE.",
                "Name the value 'card_top_spacing' in PascalCase: a val object of a composable or theme file " +
                    "(a Dp, a Color, a model instance) is PascalCase.",
                "Name the value 'CARD_SPACING' in PascalCase: a val object of a composable or theme file " +
                    "(a Dp, a Color, a model instance) is PascalCase.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_theme_object_with_miscased_value_WHEN_linted_THEN_reports_it() {
        val code = """
            package com.example.paywall.impl.theme

            internal object PaywallMotion {
                val duration = 400
                val Easing = 1
            }
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_other_files_WHEN_linted_THEN_reports_nothing() {
        val modelCode = """
            package com.example.paywall.impl.model

            private val defaultName = ""
        """.trimIndent()
        val rootCode = """
            private val defaultName = ""
        """.trimIndent()

        assertEquals(0, rule.lint(modelCode).size + rule.lint(rootCode).size)
    }
}
