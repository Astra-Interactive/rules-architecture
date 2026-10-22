@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.declaration

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class PropertyAsFunctionTest {
    private val rule = PropertyAsFunction(Config.empty)

    @Test
    fun GIVEN_functions_returning_a_stored_value_WHEN_linted_THEN_reports_each() {
        val code = """
            val defaultName = "jail"

            fun name() = defaultName

            class Jail(val id: Int) {
                private val cells = listOf(1)

                fun cells(): List<Int> {
                    return cells
                }

                fun id() = this.id
            }

            object Registry {
                private val jails = 1

                fun jails() = jails
            }
        """.trimIndent()

        val findings = rule.lint(code)

        assertEquals(4, findings.size)
        assertEquals("'name' only returns the property 'defaultName': make it a val.", findings.first().message)
    }

    @Test
    fun GIVEN_computed_or_exempt_functions_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class Jail(id: Int, val other: Jail?) : Named {
                private val name = ""
                private val cells = listOf(1)

                override fun name() = name

                suspend fun loadCells() = cells

                @Composable
                fun Cells() = cells

                fun count() = cells.size

                fun first(index: Int) = cells

                fun Jail.ownCells() = cells

                fun otherName() = other.name

                fun local() = unknown

                fun log() {
                    println(name)
                }

                fun empty() {
                    return
                }

                fun block(): List<Int> {
                    val copy = cells
                    return copy
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
