@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.compose.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ThemeObjectFunctionTest {
    private val rule = ThemeObjectFunction(Config.empty)

    @Test
    fun GIVEN_function_in_a_theme_object_WHEN_linted_THEN_reports_it() {
        val code = """
            package com.example.paywall.impl.theme

            internal object PaywallMotion {
                val Duration = 400

                fun offset(progress: Float): Float = progress * 2
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Move 'offset' out of the theme object 'PaywallMotion': a theme object holds values, not functions; " +
                    "a function goes to util/ or into the composable that uses it."
            ),
            rule.lint(code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_functions_outside_theme_objects_WHEN_linted_THEN_reports_nothing() {
        val themeCode = """
            package com.example.paywall.impl.theme

            class PaywallTheme {
                fun resolve(): Int = 0
            }

            fun formatDuration(): String = ""
        """.trimIndent()
        val modelCode = """
            package com.example.paywall.impl.model

            object Plans {
                fun first(): Int = 0
            }
        """.trimIndent()
        val rootCode = """
            object Plans {
                fun first(): Int = 0
            }
        """.trimIndent()

        assertEquals(0, rule.lint(themeCode).size + rule.lint(modelCode).size + rule.lint(rootCode).size)
    }
}
