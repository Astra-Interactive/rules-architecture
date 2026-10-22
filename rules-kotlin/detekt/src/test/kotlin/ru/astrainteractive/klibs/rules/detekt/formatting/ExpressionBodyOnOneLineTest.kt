@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.formatting

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ExpressionBodyOnOneLineTest {
    private val rule = ExpressionBodyOnOneLine(Config.empty)

    @Test
    fun GIVEN_expression_body_spanning_lines_WHEN_linted_THEN_reports_each_function() {
        val code = """
            class Names(private val users: List<String>) {
                fun trimmed(): List<String> = users
                    .map { user -> user.trim() }

                fun first(): String =
                    users.first()
            }
        """.trimIndent()

        val findings = rule.lint(code)

        assertEquals(2, findings.size)
        assertEquals(
            "The expression body of 'trimmed' spans several lines: use a block body with an explicit return.",
            findings.first().message
        )
    }

    @Test
    fun GIVEN_one_line_expression_and_block_bodies_WHEN_linted_THEN_reports_nothing() {
        val code = """
            interface Source {
                fun load(): String
            }

            fun count(
                values: List<Int>
            ): Int = values.size

            fun describe(values: List<Int>): String {
                return values
                    .joinToString()
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
