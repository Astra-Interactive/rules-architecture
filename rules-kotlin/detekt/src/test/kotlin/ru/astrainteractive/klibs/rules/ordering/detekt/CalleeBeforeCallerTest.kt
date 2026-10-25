@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.ordering.detekt

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

internal class CalleeBeforeCallerTest {
    private val rule = CalleeBeforeCaller(Config.empty)

    @Test
    fun GIVEN_member_calling_function_declared_below_WHEN_linted_THEN_reports_caller() {
        val code = """
            class OrderPricing(private val taxRate: Double) {
                fun priceOf(subtotal: Double): Double = applyTax(subtotal) + this.round(subtotal)

                private fun applyTax(subtotal: Double): Double = subtotal * (1 + taxRate)

                private fun round(value: Double): Double = value
            }
        """.trimIndent()

        val findings = rule.lint(code)

        assertEquals(2, findings.size)
        assertTrue(findings.all { finding -> finding.message.startsWith("'priceOf' calls") })
    }

    @Test
    fun GIVEN_callee_declared_above_caller_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class OrderPricing(private val taxRate: Double) {
                private val cache: MutableMap<Int, Double> = resolveCache()

                private fun resolveCache(): MutableMap<Int, Double> = mutableMapOf()

                private fun applyTax(subtotal: Double): Double = subtotal * (1 + taxRate)

                fun priceOf(subtotal: Double): Double = applyTax(subtotal)
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_top_level_extension_declared_below_WHEN_linted_THEN_reports_caller() {
        val code = """
            fun describe(price: Double): String = price.format()

            private fun Double.format(): String = toString()
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_call_on_other_receiver_with_sibling_name_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class Report(private val source: Source) {
                fun render(): String = source.load()

                private fun load(): String = ""
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_recursion_and_mutual_recursion_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun isEven(value: Int): Boolean = value == 0 || isOdd(value - 1)

            fun isOdd(value: Int): Boolean = value != 0 && isEven(value - 1)

            fun factorial(value: Int): Int = if (value <= 1) 1 else value * factorial(value - 1)
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_overload_declared_above_caller_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun format(value: Int): String = value.toString()

            fun describe(value: Int): String = format(value.toLong())

            fun format(value: Long): String = value.toString()
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_call_through_invoke_and_lambda_WHEN_linted_THEN_ignores_calls_without_simple_callee() {
        val code = """
            fun run(actions: List<() -> Unit>) = actions.first()()

            fun helper() = Unit
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
