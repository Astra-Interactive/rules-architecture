@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.naming

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ImplementationNameTest {
    private val rule = ImplementationName(Config.empty)

    @Test
    fun GIVEN_impl_suffix_WHEN_linted_THEN_reports_it() {
        val code = """
            interface JailRepository

            class JailRepositoryImpl : JailRepository

            object ClockImpl
        """.trimIndent()

        assertEquals(2, rule.lint(code).size)
    }

    @Test
    fun GIVEN_interface_suffix_WHEN_linted_THEN_reports_it() {
        val code = """
            interface RepositoryInterface
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_interface_with_i_prefix_WHEN_linted_THEN_reports_it() {
        val code = """
            interface IRepository
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_i_prefix_on_class_or_acronym_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class IRepository

            interface IOStream

            interface Item
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_names_by_technology_WHEN_linted_THEN_reports_nothing() {
        val code = """
            interface JailRepository

            class FileJailRepository : JailRepository

            val anonymous = object : JailRepository {}
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
