@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.ordering

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class DeclarationOrderTest {
    private val rule = DeclarationOrder(Config.empty)

    @Test
    fun GIVEN_class_in_the_rule_order_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class RatingComponent(private val repository: Repository) {
                private val state = mutableListOf<String>()

                constructor() : this(Repository())

                private fun load() = Unit

                class Nested

                init {
                    load()
                }

                companion object {
                    const val LIMIT = 10
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_init_block_before_functions_and_property_after_them_WHEN_linted_THEN_reports_each() {
        val code = """
            class RatingComponent {
                init {
                    load()
                }

                private fun load() = Unit

                private val ratings = listOf<Int>()
            }
        """.trimIndent()

        val messages = rule.lint(code).map { finding -> finding.message }

        assertEquals(
            listOf(
                "'load' goes before init blocks: $ORDER",
                "'ratings' goes before init blocks: $ORDER",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_companion_before_init_and_top_level_property_after_function_WHEN_linted_THEN_reports_each() {
        val code = """
            fun describe(): String = NAME

            val NAME = "rating"

            class Holder {
                companion object {
                    const val LIMIT = 1
                }

                init {
                    println()
                }
            }
        """.trimIndent()

        val messages = rule.lint(code).map { finding -> finding.message }.sorted()

        assertEquals(
            listOf("'NAME' goes before functions: $ORDER", "The init block goes before the companion object: $ORDER"),
            messages
        )
    }

    private companion object {
        const val ORDER = "properties, secondary constructors, functions, init blocks, then the companion object."
    }
}
