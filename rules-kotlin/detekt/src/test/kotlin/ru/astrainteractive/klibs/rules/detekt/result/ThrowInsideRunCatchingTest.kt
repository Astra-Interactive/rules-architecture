@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.result

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ThrowInsideRunCatchingTest {
    private val rule = ThrowInsideRunCatching(Config.empty)

    @Test
    fun GIVEN_throw_inside_catching_lambdas_WHEN_linted_THEN_reports_each() {
        val code = """
            fun load(raw: String): Result<Int> {
                return runCatching { raw.toInt() }
                    .mapCatching { value -> if (value < 0) throw IllegalStateException("negative") else value }
                    .recoverCatching { error -> listOf(1).map { throw OrderLoadError(error) }.first() }
            }
        """.trimIndent()

        val messages = rule.lint(code).map { finding -> finding.message }

        assertEquals(
            listOf(
                "Do not throw inside mapCatching: return the value, and turn the failure into a domain error with " +
                    "fold after it.",
                "Do not throw inside recoverCatching: return the value, and turn the failure into a domain error " +
                    "with fold after it.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_throw_outside_catching_lambdas_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun load(raw: String): Int {
                runCatching {
                    fun parse(): Int = throw IllegalStateException()
                    parse()
                }
                listOf(1).forEach { value -> if (value < 0) throw IllegalStateException() }
                throw IllegalStateException()
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
