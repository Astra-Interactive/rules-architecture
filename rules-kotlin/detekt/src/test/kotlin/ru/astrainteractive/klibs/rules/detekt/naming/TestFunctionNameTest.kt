@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.naming

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class TestFunctionNameTest {
    private val rule = TestFunctionName(Config.empty)

    @Test
    fun GIVEN_test_functions_with_other_names_WHEN_linted_THEN_reports_each() {
        val code = """
            import org.junit.jupiter.api.Test

            class OrderTest {
                @Test
                fun createsOrder() = Unit

                @Test
                fun `creates an order`() = Unit

                @Test
                fun GIVEN_Cart_WHEN_order_THEN_saved() = Unit

                @Test
                fun GIVEN_cart_THEN_saved() = Unit
            }
        """.trimIndent()

        assertEquals(4, rule.lint(code).size)
    }

    @Test
    fun GIVEN_test_function_in_given_when_then_form_WHEN_linted_THEN_reports_nothing() {
        val code = """
            import kotlin.test.Test

            class OrderTest {
                @Test
                fun GIVEN_empty_cart_WHEN_order_is_created_THEN_returns_cart_is_empty() = Unit
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_function_without_test_annotation_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class OrderFixture {
                fun createOrder() = Unit
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
