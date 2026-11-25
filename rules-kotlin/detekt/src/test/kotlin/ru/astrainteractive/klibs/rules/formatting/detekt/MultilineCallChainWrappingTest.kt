@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.formatting.detekt

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class MultilineCallChainWrappingTest {
    private val rule = MultilineCallChainWrapping(Config.empty)

    private fun lintOperators(code: String): List<String> {
        return rule.lint(code).map { finding ->
            val location = finding.entity.location.text
            finding.entity.ktElement.containingFile.text.substring(location.start, location.end)
        }
    }

    @Test
    fun GIVEN_call_chain_with_inline_call_and_elvis_WHEN_linted_THEN_reports_both_continuations() {
        val code = """
            val principal = principals.getFlow().first { state -> state != Principal.Loading }
                as? Principal.Token ?: return@launch
        """.trimIndent()

        assertEquals(listOf(".", "?:"), lintOperators(code))
    }

    @Test
    fun GIVEN_call_chain_with_cast_and_elvis_on_separate_lines_WHEN_linted_THEN_reports_nothing() {
        val code = """
            val principal = principals.getFlow()
                .first { state -> state != Principal.Loading }
                as? Principal.Token
                ?: return@launch
        """.trimIndent()

        assertEquals(emptyList(), lintOperators(code))
    }

    @Test
    fun GIVEN_each_call_cast_and_elvis_breaks_a_wrapped_chain_WHEN_linted_THEN_reports_only_inline_continuations() {
        val code = """
            val value = source
                .first()
                .second()
                .third() as? Token
                ?: fallback()
        """.trimIndent()

        assertEquals(listOf("as?"), lintOperators(code))
    }

    @Test
    fun GIVEN_safe_calls_follow_a_wrapped_chain_WHEN_linted_THEN_requires_each_safe_call_on_its_own_line() {
        val code = """
            val value = source
                ?.first()
                ?.second()
                ?.third()
                ?: fallback()
        """.trimIndent()

        assertEquals(emptyList(), lintOperators(code))
    }

    @Test
    fun GIVEN_safe_call_chain_continues_inline_after_a_line_break_WHEN_linted_THEN_reports_the_continuation() {
        val code = """
            val value = source?.first()
                ?.second()?.third()
        """.trimIndent()

        assertEquals(listOf("?."), lintOperators(code))
    }

    @Test
    fun GIVEN_wrapped_property_prefix_with_calls_WHEN_linted_THEN_properties_may_share_a_line_with_the_first_call() {
        val code = """
            val value = source
                .account.load()
                .first()
                as? Token
                ?: fallback()
        """.trimIndent()

        assertEquals(emptyList(), lintOperators(code))
    }

    @Test
    fun GIVEN_wrapped_property_prefix_before_first_call_WHEN_linted_THEN_later_calls_continue_on_new_lines() {
        val code = """
            val value = source
                .account.load().first()
        """.trimIndent()

        assertEquals(listOf("."), lintOperators(code))
    }

    @Test
    fun GIVEN_wrapped_first_call_and_inline_second_call_WHEN_linted_THEN_reports_the_second_call() {
        val code = """
            val value = source
                .first().second()
        """.trimIndent()

        assertEquals(listOf("."), lintOperators(code))
    }

    @Test
    fun GIVEN_plain_function_chain_is_on_one_line_WHEN_linted_THEN_reports_nothing() {
        val code = """
            val value = loadValue().normalize()
        """.trimIndent()

        assertEquals(emptyList(), lintOperators(code))
    }

    @Test
    fun GIVEN_plain_function_is_followed_by_wrapped_cast_and_inline_elvis_WHEN_linted_THEN_reports_the_elvis() {
        val code = """
            val value = loadValue()
                as? Token ?: return
        """.trimIndent()

        assertEquals(listOf("?:"), lintOperators(code))
    }

    @Test
    fun GIVEN_chain_wraps_only_before_elvis_WHEN_linted_THEN_reports_inline_calls_and_cast() {
        val code = """
            val value = source.first().second() as? Token
                ?: fallback()
        """.trimIndent()

        assertEquals(listOf(".", "as?"), lintOperators(code))
    }

    @Test
    fun GIVEN_plain_function_starts_a_wrapped_chain_WHEN_following_call_is_inline_THEN_reports_the_continuation() {
        val code = """
            val value = loadValue()
                .normalize().trim()
        """.trimIndent()

        assertEquals(listOf("."), lintOperators(code))
    }

    @Test
    fun GIVEN_call_chain_is_on_one_line_WHEN_linted_THEN_reports_nothing() {
        val code = """
            val value = source.first().second() as? Token ?: fallback()
        """.trimIndent()

        assertEquals(emptyList(), lintOperators(code))
    }

    @Test
    fun GIVEN_newlines_are_inside_arguments_and_lambda_bodies_WHEN_linted_THEN_reports_nothing() {
        val code = """
            val value = source.map(
                transform = { item ->
                    item.normalize()
                }
            ).first()

            val other = source.map { item ->
                item.normalize()
            }.first()
        """.trimIndent()

        assertEquals(emptyList(), lintOperators(code))
    }

    @Test
    fun GIVEN_cast_and_elvis_without_a_call_chain_WHEN_linted_THEN_reports_nothing() {
        val code = """
            val value = input as? Token ?: fallback()
            val other = input
                as? Token ?: fallback()
        """.trimIndent()

        assertEquals(emptyList(), lintOperators(code))
    }

    @Test
    fun GIVEN_nested_lambda_has_its_own_wrapped_chain_WHEN_linted_THEN_reports_its_inline_call_continuation() {
        val code = """
            val values = source.map { item ->
                item
                    .normalize().trim()
            }
        """.trimIndent()

        assertEquals(listOf("."), lintOperators(code))
    }

    @Test
    fun GIVEN_parenthesized_chain_contains_an_independently_wrapped_chain_WHEN_linted_THEN_reports_its_continuation() {
        val code = """
            val value = (source.first()
                .second().third())
                .normalize()
        """.trimIndent()

        assertEquals(listOf("."), lintOperators(code))
    }

    @Test
    fun GIVEN_comments_separate_a_continuation_operator_from_its_expression_WHEN_linted_THEN_counts_the_line_break() {
        val code = """
            val value = source
                .first() /* continued inline */.second()
        """.trimIndent()

        assertEquals(listOf("."), lintOperators(code))
    }

    @Test
    fun GIVEN_comment_only_line_separates_a_continuation_operator_WHEN_linted_THEN_reports_nothing() {
        val code = """
            val value = source
                .first()
                // the next call continues the chain
                .second()
        """.trimIndent()

        assertEquals(emptyList(), lintOperators(code))
    }

    @Test
    fun GIVEN_chain_operators_are_separated_by_crlf_WHEN_linted_THEN_recognizes_the_line_break() {
        val code = """
            val value = source
                .first().second()
        """.trimIndent().replace("\n", "\r\n")

        assertEquals(listOf("."), lintOperators(code))
    }

    @Test
    fun GIVEN_chain_examples_in_nested_and_companion_scopes_WHEN_linted_THEN_reports_each_inline_continuation() {
        val code = """
            class Outer {
                class Nested {
                    val nestedValue = source
                        .first().second()
                }

                companion object {
                    val companionValue = source
                        .first().second()
                }
            }

            fun Source.render(): String = normalize()
                .trim().uppercase()
        """.trimIndent()

        assertEquals(
            listOf(
                ".",
                ".",
                "."
            ),
            lintOperators(code)
        )
    }

    @Test
    fun GIVEN_expect_and_actual_declarations_without_bodies_WHEN_linted_THEN_reports_nothing() {
        val code = """
            expect fun loadValue(): String
            actual fun loadValue(): String
        """.trimIndent()

        assertEquals(emptyList(), lintOperators(code))
    }

    @Test
    fun GIVEN_generated_comment_and_test_function_contain_wrapped_chains_WHEN_linted_THEN_checks_source_expressions() {
        val code = """
            // @generated
            fun generatedValue(): String {
                return source
                    .first().second()
            }

            fun GIVEN_sample_WHEN_called_THEN_returns_value(): String {
                return source
                    .first().second()
            }
        """.trimIndent()

        assertEquals(listOf(".", "."), lintOperators(code))
    }

    @Test
    fun GIVEN_line_break_after_a_call_operator_WHEN_linted_THEN_reports_inline_continuation_operators() {
        val code = """
            val value = source.first().second().
                third()
        """.trimIndent()

        assertEquals(listOf(".", "."), lintOperators(code))
    }

    @Test
    fun GIVEN_unfinished_call_cast_and_elvis_WHEN_linted_THEN_checks_available_continuations() {
        assertEquals(listOf("?:"), lintOperators("val value = source.first()\n .second() ?:"))
        assertEquals(listOf("as?"), lintOperators("val value = source.first()\n .second() as?"))
        assertEquals(emptyList(), lintOperators("val value = source.first()."))
        assertEquals(emptyList(), lintOperators("val value = ?: fallback()"))
    }

    @Test
    fun GIVEN_other_binary_type_and_callable_reference_expressions_WHEN_linted_THEN_reports_nothing() {
        val code = """
            val same = first + second
            val condition = first == second
            val valid = value is Token
            val converted = value as Token
            val reference = String::trim
            val negative = -value
        """.trimIndent()

        assertEquals(emptyList(), lintOperators(code))
    }
}
