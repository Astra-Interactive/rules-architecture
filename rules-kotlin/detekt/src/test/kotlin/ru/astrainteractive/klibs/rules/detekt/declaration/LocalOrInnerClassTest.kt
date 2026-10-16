@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.declaration

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class LocalOrInnerClassTest {
    private val rule = LocalOrInnerClass(Config.empty)

    @Test
    fun GIVEN_class_inside_function_body_WHEN_linted_THEN_reports_it() {
        val code = """
            fun total(): Int {
                class Accumulator(var sum: Int)
                return Accumulator(0).sum
            }
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_inner_class_WHEN_linted_THEN_reports_it() {
        val code = """
            class Outer(val value: Int) {
                inner class Reader {
                    fun read(): Int = value
                }
            }
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_nested_class_and_object_expression_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class Outer {
                class Nested(val value: Int)

                fun runnable(): Runnable = object : Runnable {
                    override fun run() = Unit
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
