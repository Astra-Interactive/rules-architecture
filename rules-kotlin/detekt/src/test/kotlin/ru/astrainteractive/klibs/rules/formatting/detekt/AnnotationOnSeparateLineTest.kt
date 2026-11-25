package ru.astrainteractive.klibs.rules.formatting.detekt

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class AnnotationOnSeparateLineTest {
    private val rule = AnnotationOnSeparateLine(Config.empty)

    @Test
    fun GIVEN_inline_constructor_property_annotation_WHEN_linted_THEN_reports_violation() {
        assertEquals(1, rule.lint("data class Entry(@SerialName(\"index\") val index: Int)").size)
    }

    @Test
    fun GIVEN_annotation_above_constructor_property_WHEN_linted_THEN_reports_nothing() {
        val code = """
            data class Entry(
                @SerialName("index")
                val index: Int
            )
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_two_annotations_on_one_line_WHEN_linted_THEN_reports_violation() {
        assertEquals(2, rule.lint("class Entry @First @Second constructor() ").size)
    }

    @Test
    fun GIVEN_separate_annotations_above_declaration_WHEN_linted_THEN_reports_nothing() {
        val code = """
            @First
            @Second
            class Entry
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_multiline_annotation_ending_on_declaration_line_WHEN_linted_THEN_reports_violation() {
        val code = """
            class Entry(
                @SerialName(
                    "index"
                ) val index: Int
            )
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_inline_function_and_parameter_annotations_WHEN_linted_THEN_reports_both() {
        val code = """
            @First fun load(@Second value: String) = value
        """.trimIndent()

        assertEquals(2, rule.lint(code).size)
    }

    @Test
    fun GIVEN_use_site_annotations_inline_with_property_WHEN_linted_THEN_reports_violation() {
        val code = """
            class Entry(@field:First @get:Second val index: Int)
        """.trimIndent()

        assertEquals(2, rule.lint(code).size)
    }

    @Test
    fun GIVEN_type_use_and_expression_annotations_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun load(value: @First String): String = (@Second value)
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_nested_companion_and_extension_declarations_with_inline_annotations_WHEN_linted_THEN_reports_three() {
        val code = """
            class Outer {
                class Nested { @First val value = 1 }
                companion object { @Second fun create() = Outer() }
            }
            @Third fun String.asValue() = this
        """.trimIndent()

        assertEquals(3, rule.lint(code).size)
    }

    @Test
    fun GIVEN_inline_actual_declaration_annotation_WHEN_linted_THEN_reports_violation() {
        val code = """
            actual class PlatformEntry @First constructor()
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_same_line_comment_between_annotation_and_declaration_WHEN_linted_THEN_reports_violation() {
        val code = """
            @First /* comment */ class Entry
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_comment_after_annotation_before_next_line_declaration_WHEN_linted_THEN_reports_nothing() {
        val code = """
            @First /* comment
            */
            class Entry
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_public_modifier_before_inline_annotation_WHEN_linted_THEN_reports_violation() {
        val code = """
            public @First
            class Entry
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_annotation_above_public_class_WHEN_linted_THEN_reports_nothing() {
        val code = """
            @First
            public class Entry
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_annotation_before_expect_modifier_on_same_line_WHEN_linted_THEN_reports_nothing() {
        val code = """
            @First
            expect class Entry
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
