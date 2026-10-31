@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.formatting.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class GuardIfBracesTest {
    private val rule = GuardIfBraces(Config.empty)

    @Test
    fun GIVEN_guards_in_braces_that_fit_one_line_WHEN_linted_THEN_reports_each() {
        val code = """
            fun hide(isVisible: Boolean) {
                LaunchedEffect(isVisible) {
                    if (!isVisible) {
                        return@LaunchedEffect
                    }
                    onHide.invoke()
                }
            }

            fun price(order: Order?): Int {
                if (order == null) {
                    return 0
                }
                return order.total
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Write the guard on one line without braces: 'if (!isVisible) return@LaunchedEffect'.",
                "Write the guard on one line without braces: 'if (order == null) return 0'.",
            ),
            rule.lint(code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_one_line_guards_longer_bodies_and_long_conditions_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun price(order: Order?, discount: Discount?): Int {
                if (order == null) return 0
                if (discount == null) {
                    log("no discount")
                    return order.total
                }
                if (order.items.isEmpty() && discount.isSeasonal && discount.validUntil > order.createdAt.plus(1)) {
                    return order.totalWithoutAnyDiscountApplied
                }
                return if (order.isPaid) {
                    0
                } else {
                    order.total
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
