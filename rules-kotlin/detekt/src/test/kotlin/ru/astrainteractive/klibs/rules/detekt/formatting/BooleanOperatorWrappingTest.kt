@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.formatting

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class BooleanOperatorWrappingTest {
    private val rule = BooleanOperatorWrapping(Config.empty)

    @Test
    fun GIVEN_condition_wrapped_before_and_after_operators_WHEN_linted_THEN_reports_each_operator() {
        val code = """
            fun isReady(isBusy: Boolean, isShown: Boolean, isAlive: Boolean): Boolean {
                if (!isBusy &&
                    isShown
                    || isAlive
                ) {
                    return true
                }
                return false
            }
        """.trimIndent()

        val findings = rule.lint(code)

        assertEquals(2, findings.size)
        assertEquals(
            "The condition is wrapped at '&&': split it into guard clauses or named vals, or chain the named " +
                "values with .and()/.or(), one step per line.",
            findings.first().message
        )
    }

    @Test
    fun GIVEN_conditions_on_one_line_and_other_wrapped_operators_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun isReady(isBusy: Boolean, isShown: Boolean, count: Int): Boolean {
                val total = count +
                    1
                return !isBusy && isShown || total > 1
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
