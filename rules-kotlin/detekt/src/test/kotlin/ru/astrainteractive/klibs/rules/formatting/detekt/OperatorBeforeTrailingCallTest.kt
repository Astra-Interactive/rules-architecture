@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.formatting.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class OperatorBeforeTrailingCallTest {
    private val rule = OperatorBeforeTrailingCall(Config.empty)

    @Test
    fun GIVEN_arithmetic_before_trailing_calls_WHEN_linted_THEN_reports_each() {
        val code = """
            fun lineHeight(size: Float, scale: Float, offset: Float): List<Any> = listOf(
                (size * scale).sp,
                (size + scale * offset).toInt(),
                size.times(scale).plus(offset).minus(1f)
            )
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

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

        assertEquals(0, rule.lint(code).size)
    }
}
