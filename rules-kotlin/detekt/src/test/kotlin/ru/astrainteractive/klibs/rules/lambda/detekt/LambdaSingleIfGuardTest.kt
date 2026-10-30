@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.lambda.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class LambdaSingleIfGuardTest {
    private val rule = LambdaSingleIfGuard(Config.empty)

    @Test
    fun GIVEN_lambdas_of_one_if_WHEN_linted_THEN_reports_each() {
        val code = """
            fun render(stage: Int, items: List<Int>) {
                modifier.pointerInput(stage) {
                    if (stage == 1) {
                        detectTapGestures()
                    }
                }
                val action = { if (stage > 0) println(stage) }
                items.forEach { item -> if (item > 0) println(item) }
            }
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(3, messages.size)
        assertEquals(
            "Invert the condition into a guard, 'if (!condition) return@pointerInput', and put the body after it: " +
                "a lambda whose whole body is one if is a guard.",
            messages.first()
        )
        assertEquals(
            "Invert the condition into a guard, 'if (!condition) return', and put the body after it: a lambda " +
                "whose whole body is one if is a guard.",
            messages[1]
        )
    }

    @Test
    fun GIVEN_guards_if_else_and_longer_lambdas_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun render(stage: Int, items: List<Int>) {
                modifier.pointerInput(stage) {
                    if (stage != 1) {
                        return@pointerInput
                    }
                    detectTapGestures()
                }
                items.map { item -> if (item > 0) item else 0 }
                items.forEach { item ->
                    if (item > 0) println(item)
                    println(item)
                }
                items.forEach { }
                items.forEach { item -> if (item < 0) throw IllegalStateException("Negative item") }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
