@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.naming.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class FunctionVerbTest {
    private val rule = FunctionVerb(Config.empty)

    @Test
    fun GIVEN_functions_with_forbidden_verbs_WHEN_linted_THEN_reports_each() {
        val code = """
            fun getUser(): String = ""
            fun mapToContent(): String = ""
            fun handleClick() = Unit
            fun doWork() = Unit
            fun processOrder() = Unit
            fun makeFoo() = Unit
            fun toContentMap() = Unit
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(7, messages.size)
        assertEquals(
            "Name 'getUser' with a verb that says what happens instead of 'get': find, load, resolve, build, " +
                "compute or a val.",
            messages.first()
        )
        assertEquals(
            "Name 'mapToContent' without 'map': use a verb that says what happens, such as to<Type>, on<Event>, " +
                "build, update or save.",
            messages[1]
        )
    }

    @Test
    fun GIVEN_allowed_names_and_exempt_functions_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class State : Source {
                override fun getValue(): String = ""

                operator fun get(index: Int): Int = index

                fun getState(): Int = 0

                fun provideHandler(): Int = 0

                fun findMapping(): Int = 0

                fun loadDomain(): Int = 0

                fun redo(): Int = 0

                fun target(): Int = 0

                @Composable
                fun getColor(): Int = 0

                @Test
                fun GIVEN_get_WHEN_mapped_THEN_done() = Unit
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
