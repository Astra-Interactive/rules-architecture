@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.formatting

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ArgumentPerLineTest {
    private val rule = ArgumentPerLine(Config.empty)

    @Test
    fun GIVEN_calls_with_three_arguments_sharing_a_line_WHEN_linted_THEN_reports_each_call() {
        val code = """
            open class Base(val first: Int, val second: Int, val third: Int)

            class Child : Base(1, 2, 3)

            fun copy(state: Map<String, Int>) = state.plus(listOf("a" to 1, "b" to 2, "c" to 3))

            fun build() = listOf(
                1, 2,
                3
            )
        """.trimIndent()

        val findings = rule.lint(code)

        assertEquals(3, findings.size)
        assertEquals("'Base' takes 3 arguments: put each argument on its own line.", findings.first().message)
    }

    @Test
    fun GIVEN_arguments_on_own_lines_or_fewer_than_three_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun build(values: List<Int>) = listOf(
                1,
                // the second value
                2,
                3
            )

            fun pair(values: List<Int>) = values.fold(0, Int::plus)

            fun run(values: List<Int>) = values.forEachIndexed { index, value -> println(index + value) }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
