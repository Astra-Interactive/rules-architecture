@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.lambda.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import dev.detekt.test.utils.compileForTest
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.io.File
import kotlin.test.assertEquals

internal class ExplicitItNameTest {
    private val rule = ExplicitItName(Config.empty)

    private fun lintFindings(code: String): List<String> {
        return rule.lint(code).map { finding ->
            val location = finding.entity.location
            val reported = code.substring(location.text.start, location.text.end)
            "$reported at ${location.source.line}:${location.source.column} ${finding.message}"
        }
    }

    @Test
    fun GIVEN_documented_from_dto_with_parameter_it_WHEN_linted_THEN_reports_the_parameter_name_once() {
        val code = """
            fun fromDTO(it: WordDTO): Word = Word(it.value)
        """.trimIndent()

        assertEquals(listOf("it at 1:13 Name the parameter instead of `it`."), lintFindings(code))
    }

    @Test
    fun GIVEN_constructor_property_named_it_WHEN_linted_THEN_reports_the_parameter() {
        val code = """
            class Word(val it: String)
        """.trimIndent()

        assertEquals(listOf("it at 1:16 Name the parameter instead of `it`."), lintFindings(code))
    }

    @Test
    fun GIVEN_explicit_lambda_parameter_it_WHEN_linted_THEN_reports_the_parameter() {
        val code = """
            fun lengths(words: List<String>) = words.map { it -> it.length }
        """.trimIndent()

        assertEquals(listOf("it at 1:48 Name the parameter instead of `it`."), lintFindings(code))
    }

    @Test
    fun GIVEN_typed_lambda_parameter_it_WHEN_linted_THEN_reports_the_parameter() {
        val code = """
            fun lengths(words: List<String>) = words.map { it: String -> it.length }
        """.trimIndent()

        assertEquals(listOf("it at 1:48 Name the parameter instead of `it`."), lintFindings(code))
    }

    @Test
    fun GIVEN_destructuring_declaration_entry_it_WHEN_linted_THEN_reports_the_entry() {
        val code = """
            fun describe(entry: Map.Entry<String, Int>): String {
                val (it, count) = entry
                return it + count
            }
        """.trimIndent()

        assertEquals(listOf("it at 2:10 Name the entry instead of `it`."), lintFindings(code))
    }

    @Test
    fun GIVEN_destructured_lambda_entry_it_WHEN_linted_THEN_reports_the_entry() {
        val code = """
            fun describe(counts: Map<String, Int>) = counts.map { (it, count) -> it + count }
        """.trimIndent()

        assertEquals(listOf("it at 1:56 Name the entry instead of `it`."), lintFindings(code))
    }

    @Test
    fun GIVEN_local_property_named_it_WHEN_linted_THEN_reports_the_property() {
        val code = """
            fun total(prices: List<Int>): Int {
                val it = prices.sum()
                return it
            }
        """.trimIndent()

        assertEquals(listOf("it at 2:9 Name the property instead of `it`."), lintFindings(code))
    }

    @Test
    fun GIVEN_member_property_named_it_WHEN_linted_THEN_reports_the_property() {
        val code = """
            class Cart(prices: List<Int>) {
                val it = prices.sum()
            }
        """.trimIndent()

        assertEquals(listOf("it at 2:9 Name the property instead of `it`."), lintFindings(code))
    }

    @Test
    fun GIVEN_for_loop_variable_it_WHEN_linted_THEN_reports_the_loop_variable() {
        val code = """
            fun total(prices: List<Int>): Int {
                var sum = 0
                for (it in prices) sum += it
                return sum
            }
        """.trimIndent()

        assertEquals(listOf("it at 3:10 Name the loop variable instead of `it`."), lintFindings(code))
    }

    @Test
    fun GIVEN_catch_parameter_it_WHEN_linted_THEN_reports_the_parameter() {
        val code = """
            fun parse(raw: String): Int? {
                return try {
                    raw.toInt()
                } catch (it: NumberFormatException) {
                    null
                }
            }
        """.trimIndent()

        assertEquals(listOf("it at 4:14 Name the parameter instead of `it`."), lintFindings(code))
    }

    @Test
    fun GIVEN_function_class_and_type_parameter_named_it_WHEN_linted_THEN_reports_each_declaration() {
        val code = """
            fun it(): Int = 1

            class it

            fun <it> first(values: List<it>): it = values.first()
        """.trimIndent()

        assertEquals(
            listOf(
                "it at 1:5 Name the declaration instead of `it`.",
                "it at 3:7 Name the declaration instead of `it`.",
                "it at 5:6 Name the declaration instead of `it`."
            ),
            lintFindings(code)
        )
    }

    @Test
    fun GIVEN_function_type_parameter_named_it_WHEN_linted_THEN_reports_the_parameter() {
        val code = """
            class Formatter(val format: (it: Int) -> String)
        """.trimIndent()

        assertEquals(listOf("it at 1:30 Name the parameter instead of `it`."), lintFindings(code))
    }

    @Test
    fun GIVEN_backticked_parameter_it_WHEN_linted_THEN_reports_the_parameter() {
        val code = """
            fun echo(`it`: Int): Int = `it`
        """.trimIndent()

        assertEquals(listOf("`it` at 1:10 Name the parameter instead of `it`."), lintFindings(code))
    }

    @Test
    fun GIVEN_parameter_it_of_nested_class_WHEN_linted_THEN_reports_the_parameter() {
        val code = """
            class Outer {
                class Inner(val it: Int)
            }
        """.trimIndent()

        assertEquals(listOf("it at 2:21 Name the parameter instead of `it`."), lintFindings(code))
    }

    @Test
    fun GIVEN_companion_constant_named_it_WHEN_linted_THEN_reports_the_property() {
        val code = """
            class Limits {
                companion object {
                    const val it = 10
                }
            }
        """.trimIndent()

        assertEquals(listOf("it at 3:19 Name the property instead of `it`."), lintFindings(code))
    }

    @Test
    fun GIVEN_extension_function_parameter_it_WHEN_linted_THEN_reports_the_parameter() {
        val code = """
            fun Int.shift(it: Int): Int = this + it
        """.trimIndent()

        assertEquals(listOf("it at 1:15 Name the parameter instead of `it`."), lintFindings(code))
    }

    @Test
    fun GIVEN_expect_and_actual_parameter_it_WHEN_linted_THEN_reports_both_declarations() {
        val code = """
            expect fun format(it: Int): String

            actual fun format(it: Int): String = it.toString()
        """.trimIndent()

        assertEquals(
            listOf(
                "it at 1:19 Name the parameter instead of `it`.",
                "it at 3:19 Name the parameter instead of `it`."
            ),
            lintFindings(code)
        )
    }

    @Test
    fun GIVEN_override_parameter_it_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class Printer : Sink {
                override fun accept(it: String) = Unit
            }
        """.trimIndent()

        assertEquals(emptyList(), lintFindings(code))
    }

    @Test
    fun GIVEN_supertype_and_override_parameter_it_WHEN_linted_THEN_reports_the_supertype_and_the_lambda_in_the_body() {
        val code = """
            interface Sink {
                fun accept(it: String)
            }

            class Printer : Sink {
                override fun accept(it: String) {
                    listOf(it).forEach { it -> check(it.isNotEmpty()) }
                }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "it at 2:16 Name the parameter instead of `it`.",
                "it at 7:30 Name the parameter instead of `it`."
            ),
            lintFindings(code)
        )
    }

    @Test
    fun GIVEN_override_properties_named_it_WHEN_linted_THEN_reports_only_the_supertype_property() {
        val code = """
            interface Named {
                val it: String
            }

            class Word(override val it: String) : Named

            class Label : Named {
                override val it: String = "label"
            }
        """.trimIndent()

        assertEquals(listOf("it at 2:9 Name the property instead of `it`."), lintFindings(code))
    }

    @Test
    fun GIVEN_lambda_parameter_it_in_test_sources_WHEN_linted_THEN_reports_it(@TempDir directory: File) {
        val file = File(directory, "src/test/kotlin/WordMapperTest.kt")
        file.parentFile.mkdirs()
        file.writeText(
            """
            class WordMapperTest {
                fun GIVEN_word_WHEN_mapped_THEN_keeps_value() {
                    listOf("a").forEach { it -> check(it.isNotEmpty()) }
                }
            }
            """.trimIndent()
        )

        val messages = rule.lint(compileForTest(file.toPath())).map(Finding::message)

        assertEquals(listOf("Name the parameter instead of `it`."), messages)
    }

    @Test
    fun GIVEN_item_iterator_string_and_implicit_it_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun names(items: List<String>, iterator: Iterator<String>): List<String> {
                val item = iterator.next()
                val label = "it"
                return items.map { it.trim() } + item + label
            }
        """.trimIndent()

        assertEquals(emptyList(), lintFindings(code))
    }

    @Test
    fun GIVEN_documented_bad_implicit_it_WHEN_linted_THEN_leaves_it_to_implicit_it_parameter() {
        val code = """
            fun load() {
                loadRequest().let {
                    logger.info("Loaded ${'$'}it")
                }
            }
        """.trimIndent()

        assertEquals(emptyList(), lintFindings(code))
    }

    @Test
    fun GIVEN_documented_good_named_lambda_parameter_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun load() {
                loadRequest().let { request ->
                    logger.info("Loaded ${'$'}request")
                }
            }
        """.trimIndent()

        assertEquals(emptyList(), lintFindings(code))
    }
}
