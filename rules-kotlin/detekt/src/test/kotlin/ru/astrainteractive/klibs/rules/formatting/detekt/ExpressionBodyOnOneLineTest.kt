@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.formatting.detekt

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ExpressionBodyOnOneLineTest {
    private val rule = ExpressionBodyOnOneLine(Config.empty)

    @Test
    fun GIVEN_expression_bodies_spanning_lines_WHEN_linted_THEN_reports_each_function() {
        val code = """
            class Names(private val users: List<String>) {
                fun trimmed(): List<String> = users
                    .map { user -> user.trim() }

                fun first(): String =
                    users.first()

                private fun count(
                    offset: Int
                ): Int = users.size + offset

                fun stage(value: Int): String = when {
                    value > 0 -> "positive"
                    else -> "other"
                }
            }
        """.trimIndent()

        val findings = rule.lint(code)

        assertEquals(4, findings.size)
        assertEquals(
            "The expression-body function 'trimmed' spans several lines: use a block body with an explicit " +
                "return; an expression body is one line, signature and body.",
            findings.first().message
        )
    }

    @Test
    fun GIVEN_one_line_expression_and_block_bodies_WHEN_linted_THEN_reports_nothing() {
        val code = """
            interface Source {
                fun load(): String
            }

            /**
             * Counts the values.
             */
            @Suppress("Unused")
            @JvmName("countValues")
            internal fun count(values: List<Int>): Int = values.size

            @Composable
            fun Title() = Unit

            fun describe(values: List<Int>): String {
                return values
                    .joinToString()
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
