@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.compose

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class PreviewBlockLastTest {
    private val rule = PreviewBlockLast(Config.empty)

    @Test
    fun GIVEN_declarations_after_the_first_preview_WHEN_linted_THEN_reports_each_that_is_not_for_previews() {
        val code = """
            private val CardSpacing = 4

            @Composable
            internal fun HeroCopyComposable() = Unit

            private const val PREVIEW_TITLE = "Unlock more control."

            @Composable
            private fun HeroCopyPreviewSheet() = Unit

            @Preview
            @Composable
            private fun HeroCopyCenterPreview() = Unit

            private val PreviewPlans = listOf(1)

            private val TitleSize = 12

            @Composable
            private fun HeroCopyFooter() = Unit

            @Preview
            @Composable
            private fun HeroCopyEnd() = Unit

            typealias Plans = List<Int>
        """.trimIndent()

        val messages = rule.lint(code).map { finding -> finding.message }

        assertEquals(
            listOf(
                "Name the preview 'HeroCopyEnd' <Composable><Scenario>Preview.",
                "Move 'TitleSize' above the previews, or name it for the preview block: the previews close the " +
                    "file and hold only preview functions and PREVIEW_/Preview values.",
                "Move 'HeroCopyFooter' above the previews, or name it for the preview block: the previews close " +
                    "the file and hold only preview functions and PREVIEW_/Preview values.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_file_without_previews_WHEN_linted_THEN_reports_nothing() {
        val code = """
            @Composable
            internal fun HeroCopyComposable() = Unit

            private val TitleSize = 12
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
