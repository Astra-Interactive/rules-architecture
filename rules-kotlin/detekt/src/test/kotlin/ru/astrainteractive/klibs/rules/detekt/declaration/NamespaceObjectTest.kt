@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.declaration

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class NamespaceObjectTest {
    private val rule = NamespaceObject(Config.empty)

    @Test
    fun GIVEN_object_holding_only_functions_WHEN_linted_THEN_reports_it() {
        val code = """
            object DurationFormat {
                fun format(seconds: Int): String = "${'$'}seconds s"
                fun parse(text: String): Int = text.removeSuffix(" s").toInt()
            }
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_object_with_supertype_WHEN_linted_THEN_reports_nothing() {
        val code = """
            interface Formatter {
                fun format(seconds: Int): String
            }

            object SecondsFormatter : Formatter {
                override fun format(seconds: Int): String = "${'$'}seconds s"
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_object_with_state_WHEN_linted_THEN_reports_nothing() {
        val code = """
            object Keys {
                const val LIMIT = 10
                fun limit(): Int = LIMIT
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_empty_object_WHEN_linted_THEN_reports_nothing() {
        val code = """
            object Marker
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_companion_and_object_expression_with_only_functions_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class Pager {
                companion object {
                    fun create(): Pager = Pager()
                }
            }

            val listener = object {
                fun onEvent() = Unit
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_component_defaults_object_holding_only_functions_WHEN_linted_THEN_reports_nothing() {
        val code = """
            object ButtonDefaults {
                fun colors(): ButtonColors = ButtonColors()
                fun shape(): Shape = RoundedShape()
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
