@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.declaration.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import dev.detekt.test.utils.compileContentForTest
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class OneTypeFamilyPerFileTest {
    private val rule = OneTypeFamilyPerFile(Config.empty)

    @Test
    fun GIVEN_functions_beside_a_class_WHEN_linted_THEN_reports_each_function() {
        val code = """
            class Pricing

            fun format(value: Int): String = value.toString()

            @Composable
            fun PricingCard() = Unit
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(
            listOf(
                "'format' sits beside the type 'Pricing': a file that declares a class holds no top-level " +
                    "function; make it a private member or an extension in util/<Subject>Ext.kt.",
                "'PricingCard' is a composable beside the type 'Pricing': a composable file declares no class; " +
                    "move 'Pricing' to its own file.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_two_classes_with_behaviour_WHEN_linted_THEN_reports_the_second() {
        val code = """
            class CachedRepository {
                fun load() = Unit
            }

            abstract class RefreshingCache

            data class Entry(val id: Int)
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(
            listOf(
                "Move 'RefreshingCache' to its own file: a file holds one type family.",
                "Move 'Entry' to its own file: a file holds one type family."
            ),
            messages
        )
    }

    @Test
    fun GIVEN_one_type_family_per_file_WHEN_linted_THEN_reports_nothing() {
        val code = """
            sealed class OrderError(message: String) : Exception(message)

            class OrderParseError(message: String) : OrderError(message)

            class OrderLoadError(message: String) : OrderError(message)

            object Unknown : OrderError("unknown")
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_unrelated_data_and_behavior_types_WHEN_linted_THEN_reports_each_extra_family() {
        val code = """
            data class Order(val id: Int)

            value class OrderId(val value: Int)

            enum class Stage { OFFER }

            interface Source
        """.trimIndent()

        assertEquals(3, rule.lint(code).size)
    }

    @Test
    fun GIVEN_file_of_functions_only_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun format(value: Int): String = value.toString()

            fun parse(value: String): Int = value.toInt()
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_component_defaults_beside_its_composable_WHEN_linted_THEN_reports_only_the_other_type() {
        val code = """
            object GlassButtonDefaults {
                fun colors(): Int = 0
            }

            class GlassButtonState

            @Composable
            fun GlassButtonComposable() = Unit
        """.trimIndent()

        val file = compileContentForTest(code, filename = "GlassButtonComposable.kt")
        val messages = rule.lint(file).map(Finding::message)

        assertEquals(
            listOf(
                "'GlassButtonComposable' is a composable beside the type 'GlassButtonState': a composable file " +
                    "declares no class; move 'GlassButtonState' to its own file.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_defaults_object_in_a_file_of_another_name_WHEN_linted_THEN_reports_the_composable() {
        val code = """
            object GlassButtonDefaults

            @Composable
            fun GlassButtonComposable() = Unit
        """.trimIndent()

        val file = compileContentForTest(code, filename = "Buttons.kt")

        assertEquals(1, rule.lint(file).size)
    }
}
