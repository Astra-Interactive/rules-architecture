@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.lambda.detekt

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ImplicitItParameterTest {
    private val rule = ImplicitItParameter(Config.empty)

    @Test
    fun GIVEN_lambda_using_implicit_it_WHEN_linted_THEN_reports_each_use() {
        val code = """
            fun names(users: List<String>): List<String> = users
                .filter { it.isNotBlank() }
                .map { it.trim() + it.length }
        """.trimIndent()

        assertEquals(3, rule.lint(code).size)
    }

    @Test
    fun GIVEN_lambda_with_named_parameter_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun names(users: List<String>): List<String> = users.map { user -> user.trim() }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_implicit_it_of_outer_lambda_inside_named_inner_lambda_WHEN_linted_THEN_reports_it() {
        val code = """
            fun sums(rows: List<Int>, offsets: List<Int>) = offsets.map { rows.map { row -> row + it } }
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_member_named_it_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class Holder(val it: Int)

            fun read(holders: List<Holder>) = holders.map { holder -> holder.it }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_named_argument_called_it_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun take(it: Int) = it

            fun call(values: List<Int>) = values.map { value -> take(it = value) }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_it_outside_any_lambda_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun echo(it: Int): Int = it
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_label_named_it_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun first(values: List<Int>) = values.forEach it@{ value -> if (value > 0) return@it }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_suppressed_lambda_WHEN_linted_THEN_reports_nothing() {
        val code = """
            @Suppress("ImplicitItParameter")
            fun names(users: List<String>): List<String> = users.map { it.trim() }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
