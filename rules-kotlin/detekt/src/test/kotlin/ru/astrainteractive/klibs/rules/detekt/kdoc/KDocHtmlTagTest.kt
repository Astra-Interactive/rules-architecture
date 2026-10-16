@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.kdoc

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

internal class KDocHtmlTagTest {
    private val rule = KDocHtmlTag(Config.empty)

    @Test
    fun GIVEN_kdoc_with_html_tags_WHEN_linted_THEN_reports_once_listing_each_tag() {
        val code = """
            /**
             * Loads a user.<p>
             * Use <code>findUser</code> instead, see <a href="https://example.com">docs</a>.
             * <p>
             */
            fun getUser(id: Long): String = TODO()
        """.trimIndent()

        val findings = rule.lint(code)

        assertEquals(1, findings.size)
        assertTrue(findings.single().message.contains("<p>, <code>, </code>, <a href=\"https://example.com\">"))
    }

    @Test
    fun GIVEN_tags_only_inside_code_WHEN_linted_THEN_reports_nothing() {
        val code = """
            /**
             * Returns a `List<p>` of pages.
             *
             * ```kotlin
             * val pages: List<b> = load()
             * ```
             */
            fun pages(): List<String> = TODO()
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_markdown_kdoc_WHEN_linted_THEN_reports_nothing() {
        val code = """
            /**
             * Calculates the discount, see [Order].
             *
             * @param rate discount as a fraction in the range 0.0..1.0
             */
            fun discount(rate: Double): Double = rate

            class Order(val id: Long)
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
