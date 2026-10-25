@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.compose.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class RememberInNameTest {
    private val rule = RememberInName(Config.empty)

    @Test
    fun GIVEN_remember_functions_without_remember_WHEN_linted_THEN_reports_each() {
        val code = """
            @Composable
            private fun rememberFeatureShortcuts(): List<Int> = listOf(1)

            @Composable
            private fun rememberTitle(): String {
                return "title"
            }
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(2, messages.size)
        assertEquals(
            "'rememberFeatureShortcuts' calls no remember: call remember in its body, or name it with a verb " +
                "such as build, resolve or select.",
            messages.first()
        )
    }

    @Test
    fun GIVEN_remember_functions_that_remember_or_other_names_WHEN_linted_THEN_reports_nothing() {
        val code = """
            @Composable
            private fun rememberShortcuts(): List<Int> = remember { listOf(1) }

            @Composable
            private fun rememberProgress(): Float {
                val state = rememberScrollState()
                return state.value
            }

            @Composable
            private fun rememberedValue(): Int = 1

            interface Holder {
                fun rememberAll(): Int
            }

            fun buildShortcuts(): List<Int> = listOf(1)
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
