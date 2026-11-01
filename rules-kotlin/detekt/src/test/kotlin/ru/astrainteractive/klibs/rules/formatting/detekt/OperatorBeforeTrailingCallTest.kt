@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.formatting.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.createEnvironment
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class OperatorBeforeTrailingCallTest {
    private val rule = OperatorBeforeTrailingCall(Config.empty)
    private val environment = createEnvironment()

    @Test
    fun GIVEN_arithmetic_before_trailing_calls_WHEN_linted_THEN_reports_each() {
        val code = """
            fun lineHeight(size: Float, scale: Float, offset: Float): List<Any> = listOf(
                (size * scale).sp,
                (size + scale * offset).toInt(),
                size.times(scale).plus(offset).minus(1f)
            )
        """.trimIndent()

        val messages = rule.lintWithContext(environment, code).map(Finding::message)

        assertEquals(
            listOf(
                "Write 'size.times(scale)' instead of '(size * scale)' before the trailing call, so the chain " +
                    "reads left to right.",
                "Name the value of '(size + scale * offset)' in a val, then make the call on it.",
                "Name the value in a val instead of chaining 'size.times(scale).plus(offset).minus(1f)': two or more " +
                    "operators before a call are named first.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_plain_arithmetic_and_other_receivers_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun price(subtotal: Double, taxRate: Double, name: String, items: List<Int>): List<Any> = listOf(
                subtotal * (1 + taxRate),
                ("price: " + name).length,
                (items.map { item -> item + 1 }).size,
                (items.size - 1 downTo 1).first,
                (0..items.size - 1).last,
                (subtotal).toInt(),
                (subtotal > 1).toString(),
                subtotal.times(taxRate).toInt(),
                subtotal.plus(taxRate),
                items.size.toString()
            )
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_string_concatenation_with_named_calls_or_operators_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun message(name: String, error: String, value: Int): List<Any> = listOf(
                "Could not parse ".plus(name).plus(error),
                name.plus(": ").plus(error).length,
                (name + error).length,
                ((name + error) + value).length,
                name.plus(value).plus(value),
                name.plus(value + value).length
            )
        """.trimIndent()

        assertEquals(emptyList(), rule.lintWithContext(environment, code))
    }

    @Test
    fun GIVEN_nested_arithmetic_before_a_call_WHEN_linted_THEN_requests_a_value_for_the_whole_expression() {
        val code = """
            fun duration(stiffness: Float, dampingRatio: Float): Int {
                return ((1000f / (stiffness / 100f)) * (1f + (1f - dampingRatio))).toInt()
            }
            fun total(a: Int, b: Int, c: Int): List<Int> = listOf(
                ((a + b) * c).toInt(),
                (a * (b + c)).toInt(),
                (((a + b))).toInt()
            )
        """.trimIndent()

        assertEquals(
            listOf(
                "Name the value of '((1000f / (stiffness / 100f)) * (1f + (1f - dampingRatio)))' in a val, " +
                    "then make the call on it.",
                "Name the value of '((a + b) * c)' in a val, then make the call on it.",
                "Name the value of '(a * (b + c))' in a val, then make the call on it.",
                "Write 'a.plus(b)' instead of '(((a + b)))' before the trailing call, so the chain reads left to right."
            ),
            rule.lintWithContext(environment, code).map(Finding::message)
        )
    }
}
