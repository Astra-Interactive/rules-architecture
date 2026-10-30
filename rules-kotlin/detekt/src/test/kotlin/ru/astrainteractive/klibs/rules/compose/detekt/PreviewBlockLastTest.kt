@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.compose.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
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

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(
            listOf(
                "Name the preview 'HeroCopyEnd' <Composable><Scenario>Preview.",
                "Move 'TitleSize' above the previews, or name it for the preview block: the previews close the " +
                    "file and hold only preview functions and PREVIEW_/Preview values.",
                "Move 'HeroCopyFooter' above the previews, or name it for the preview block: the previews close " +
                    "the file and hold only preview functions and PREVIEW_/Preview values.",
                "Move 'Plans' above the previews, or name it for the preview block: the previews close the file " +
                    "and hold only preview functions and PREVIEW_/Preview values.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_preview_helpers_above_the_code_of_the_file_WHEN_linted_THEN_reports_each() {
        val code = """
            private const val PREVIEW_TITLE = "Unlock more control."

            @Composable
            private fun HeroCopyPreviewSheet() = Unit

            private val CardSpacing = 4

            @Composable
            internal fun HeroCopyComposable() = Unit

            private val PreviewPlans = listOf(1)

            @Preview
            @Composable
            private fun HeroCopyCenterPreview() = Unit
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(
            listOf(
                "Move 'PREVIEW_TITLE' down into the preview block, before the first function that uses it: what " +
                    "only previews use never sits at the top of the file.",
                "Move 'HeroCopyPreviewSheet' down into the preview block, before the first function that uses it: " +
                    "what only previews use never sits at the top of the file.",
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
