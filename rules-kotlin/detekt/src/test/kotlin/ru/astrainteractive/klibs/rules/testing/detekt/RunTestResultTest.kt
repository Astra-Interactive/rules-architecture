@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.testing.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class RunTestResultTest {
    private val rule = RunTestResult(Config.empty)

    @Test
    fun GIVEN_setup_before_returning_run_test_WHEN_linted_THEN_accepts_the_result() {
        val code = """
            @Test
            fun GIVEN_value_WHEN_checked_THEN_matches(): kotlinx.coroutines.test.TestResult {
                val expected = 2
                return runTest { assertEquals(expected, 2) }
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lint(code).map(Finding::message))
    }

    @Test
    fun GIVEN_tests_that_do_not_return_run_test_WHEN_linted_THEN_reports_each() {
        val code = """
            class OrderTest {
                @Test
                fun GIVEN_cart_WHEN_ordered_THEN_saves() = runTest { }

                @Test
                fun GIVEN_cart_WHEN_paid_THEN_charges() {
                    runTest { }
                }

                @Test
                fun GIVEN_cart_WHEN_cancelled_THEN_refunds(): TestResult {
                    val result = runTest { }
                    return result
                }
            }
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(3, messages.size)
        assertEquals(
            "Return runTest from 'GIVEN_cart_WHEN_ordered_THEN_saves': declare ': TestResult' and write " +
                "'return runTest { … }', since a test on JS must return what runTest returns.",
            messages.first()
        )
    }

    @Test
    fun GIVEN_tests_returning_run_test_or_without_it_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class OrderTest {
                @Test
                fun GIVEN_cart_WHEN_ordered_THEN_saves(): TestResult {
                    return runTest { }
                }

                @Test
                fun GIVEN_price_WHEN_formatted_THEN_rounds() {
                    assertEquals(1, 1)
                }

                fun runInScope() {
                    runTest { }
                }
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lint(code).map(Finding::message))
    }

    @Test
    fun GIVEN_typed_expression_body_WHEN_linted_THEN_asks_for_the_return() {
        val code = """
            class OrderTest {
                @Test
                fun GIVEN_cart_WHEN_paid_THEN_charges(): TestResult = runTest { }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Return runTest from 'GIVEN_cart_WHEN_paid_THEN_charges': declare ': TestResult' and write " +
                    "'return runTest { … }', since a test on JS must return what runTest returns."
            ),
            rule.lint(code).map(Finding::message)
        )
    }
}
