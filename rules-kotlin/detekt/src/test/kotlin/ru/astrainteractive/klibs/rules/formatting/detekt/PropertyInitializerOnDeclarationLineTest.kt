@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.formatting.detekt

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.jetbrains.kotlin.psi.KtProperty
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class PropertyInitializerOnDeclarationLineTest {
    private val rule = PropertyInitializerOnDeclarationLine(Config.empty)

    private fun lintPropertyNames(code: String): List<String?> {
        return rule.lint(code).map { finding -> (finding.entity.ktElement as KtProperty).name }
    }

    @Test
    fun GIVEN_initializer_on_the_line_after_its_equals_WHEN_linted_THEN_reports_the_property() {
        val code = """
            class TagsSelectorViewModel(tagsSelectorFeature: TagsSelectorFeature) : TagsSelector {
                override val model: StateFlow<TagsSelectorState> =
                    tagsSelectorFeature.state.asStateFlow()
            }
        """.trimIndent()

        val findings = rule.lint(code)

        assertEquals(1, findings.size)
        assertEquals(
            "The initializer of 'model' starts on the line after its '=': join the declaration into one line when " +
                "it fits MaxLineLength; otherwise start the initializer on the '=' line and wrap inside it: " +
                "arguments one per line, a chain one call per line with the receiver on the '=' line, a long " +
                "receiver named as its own val.",
            findings.single().message
        )
    }

    @Test
    fun GIVEN_initializer_starting_on_the_equals_line_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class OneLineViewModel(tagsSelectorFeature: TagsSelectorFeature) : TagsSelector {
                override val model: StateFlow<TagsSelectorState> = tagsSelectorFeature.state.asStateFlow()
            }

            class WrappedChainViewModel(tagsSelectorFeature: TagsSelectorFeature) : TagsSelector {
                override val model: StateFlow<TagsSelectorState> = tagsSelectorFeature
                    .state
                    .asStateFlow()
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_multi_line_initializers_starting_on_the_equals_line_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun describe(stage: Int, names: List<String>): String {
                val label = when (stage) {
                    0 -> "start"
                    else -> "other"
                }
                val values = listOf(
                    label,
                    names.first(),
                    names.last()
                )
                val joined = values
                    .filter(String::isNotBlank)
                    .joinToString()
                return joined
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_wrapped_initializers_at_every_declaration_level_WHEN_linted_THEN_reports_each() {
        val code = """
            const val LIMIT: Int =
                10

            actual val platformName: String =
                "jvm"

            fun String.render(): String {
                val trimmed =
                    trim()
                return trimmed
            }

            class Outer {
                class Nested {
                    private val names: List<String> =
                        listOf("a", "b")
                }

                companion object {
                    private val DEFAULT_NAME =
                        "outer"
                }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "LIMIT",
                "platformName",
                "trimmed",
                "names",
                "DEFAULT_NAME",
            ),
            lintPropertyNames(code)
        )
    }

    @Test
    fun GIVEN_comment_between_equals_and_initializer_WHEN_linted_THEN_reports_only_a_line_break() {
        val code = """
            val commentedBelow =
                // the first value
                1

            val commentedAfterEquals = // the second value
                2

            val inlineComment = /* the third value */ 3
        """.trimIndent()

        assertEquals(listOf("commentedBelow", "commentedAfterEquals"), lintPropertyNames(code))
    }

    @Test
    fun GIVEN_properties_without_an_initializer_WHEN_linted_THEN_reports_nothing() {
        val code = """
            expect val platformName: String

            abstract class Screen {
                abstract val title: String

                lateinit var subtitle: String

                val lazyTitle: String by lazy {
                    title.trim()
                }

                val upperTitle: String
                    get() =
                        title.uppercase()
            }

            val String.initial: Char
                get() = first()
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
