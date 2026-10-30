@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.declaration.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class CompanionFactoryFunctionTest {
    private val rule = CompanionFactoryFunction(Config.empty)

    @Test
    fun GIVEN_factories_in_companion_objects_WHEN_linted_THEN_reports_each() {
        val code = """
            data class Jail(val name: String) {
                companion object {
                    fun create(name: String): Jail = Jail(name)
                    fun createEmpty(): Jail = Jail("")
                }
            }
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(2, messages.size)
        assertEquals(
            "Move the factory 'create' out of the companion object: write a top-level create<Aggregate>() in util " +
                "returning Result, never a companion factory.",
            messages.first()
        )
    }

    @Test
    fun GIVEN_other_companion_functions_and_other_objects_WHEN_linted_THEN_reports_nothing() {
        val code = """
            enum class UnitKind {
                CONTEXT;

                companion object {
                    fun fromCliName(name: String): UnitKind? = null
                    fun offset(): Int = 0
                    fun of(name: String): UnitKind = CONTEXT
                    fun from(name: String): UnitKind = CONTEXT
                }
            }

            object Jails {
                fun create(): Int = 0
            }

            class Factory {
                fun create(): Int = 0
            }

            fun create(): Int = 0
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
