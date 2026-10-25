@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.ordering.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ConstantOutsideCompanionTest {
    private val rule = ConstantOutsideCompanion(Config.empty)

    @Test
    fun GIVEN_constants_in_a_class_body_WHEN_linted_THEN_reports_each() {
        val code = """
            class Reloader {
                const val LIMIT = 10
                private val interval = 1.minutes
                private val pattern = "x".toRegex()
                private val name = "reloader"
                private val offset = -1
                private val enabled = true
            }
        """.trimIndent()

        val findings = rule.lint(code)

        assertEquals(6, findings.size)
        assertEquals(
            "Move the constant 'LIMIT' to the companion object: class-level constants live there.",
            findings.first().message
        )
    }

    @Test
    fun GIVEN_private_object_of_constants_beside_a_type_WHEN_linted_THEN_reports_it() {
        val code = """
            private object ReloaderDefaults {
                val Interval = 1
            }

            class Reloader
        """.trimIndent()

        assertEquals(
            listOf(
                "Move the constants of 'ReloaderDefaults' to the companion object of the type that uses them: " +
                    "a private object of constants beside a type is forbidden."
            ),
            rule.lint(code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_state_computed_values_and_companion_constants_WHEN_linted_THEN_reports_nothing() {
        val code = """
            private object Holder {
                fun create() = Unit
            }

            private object Empty

            object Values {
                val LIMIT = 10
            }

            class Reloader(private val prefix: String) : Base {
                private var attempts = 0
                private val label = "${'$'}prefix-reloader"
                private val cache = mutableMapOf<Int, Int>()
                private val window = 1.minutes.inWholeSeconds
                private val sum = 1 + 2
                private val negated = !true
                private val lazyValue by lazy { 1 }
                override val name = "reloader"

                companion object {
                    const val LIMIT = 10
                    val INTERVAL = 1.minutes
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
