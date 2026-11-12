@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.readme.structure

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import java.io.File
import kotlin.test.assertEquals
import kotlin.test.assertTrue

internal class UnitReadmeTest {

    private fun unit(directory: File, isRoot: Boolean): BuildUnit {
        return BuildUnit(
            path = ":jail",
            directory = directory,
            isEntryPoint = false,
            isRoot = isRoot,
            projectDependencies = emptySet()
        )
    }

    private fun readme(vararg lines: String): String = lines.joinToString(separator = System.lineSeparator())

    private fun countReported(directory: File, readme: String): Int {
        File(directory, "README.md").writeText(readme)
        return UnitReadme().check(unit(directory, isRoot = false)).size
    }

    @Test
    fun GIVEN_readme_of_one_sentence_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val oneSentence = readme(
            "# jail",
            "",
            "Jails players for a sentence and frees them."
        )
        assertEquals(0, countReported(directory, oneSentence))
        assertEquals(0, countReported(directory, "Jails players for a sentence and frees them"))
        assertEquals(0, countReported(directory, "Formats durations, e.g. 1h 2m, for chat messages."))
        assertEquals(0, countReported(directory, "Formats durations, i.e. the time left, etc. for chat."))
    }

    @Test
    fun GIVEN_readme_of_several_sentences_or_none_WHEN_checked_THEN_reports_it(@TempDir directory: File) {
        val twoSentences = readme(
            "# jail",
            "",
            "Jails players. Frees them later!"
        )
        assertEquals(1, countReported(directory, twoSentences))
        assertEquals(1, countReported(directory, readme("# jail")))
    }

    @Test
    fun GIVEN_closing_abbreviation_before_uppercase_word_WHEN_checked_THEN_period_ends_a_sentence(
        @TempDir directory: File
    ) {
        assertEquals(expected = 1, actual = countReported(directory, "Stores ratings, etc. The next."))
        assertEquals(expected = 1, actual = countReported(directory, "Compares ratings vs. The next."))
        assertEquals(expected = 1, actual = countReported(directory, "Stores ratings, ETC. The next."))
        assertEquals(expected = 1, actual = countReported(directory, readme("Stores ratings, etc.", "The next.")))
    }

    @Test
    fun GIVEN_abbreviation_not_before_uppercase_word_WHEN_checked_THEN_counts_one_sentence(
        @TempDir directory: File
    ) {
        assertEquals(expected = 0, actual = countReported(directory, "Stores ratings, files, etc. for players."))
        assertEquals(expected = 0, actual = countReported(directory, "Stores ratings, etc."))
        assertEquals(expected = 0, actual = countReported(directory, "Stores ratings, etc.The next."))
        assertEquals(expected = 0, actual = countReported(directory, "Formats durations, e.g. Monday."))
        assertEquals(expected = 0, actual = countReported(directory, "Formats durations, i.e. The time left."))
    }

    @Test
    fun GIVEN_text_after_last_terminal_WHEN_checked_THEN_it_counts_as_a_sentence(@TempDir directory: File) {
        assertEquals(expected = 1, actual = countReported(directory, "Jails players. Frees them"))
        assertEquals(expected = 1, actual = countReported(directory, "Jails players! Frees them later? And more"))
        assertEquals(expected = 0, actual = countReported(directory, "Jails players. "))
    }

    @Test
    fun GIVEN_initialism_WHEN_checked_THEN_its_periods_do_not_end_a_sentence(@TempDir directory: File) {
        assertEquals(expected = 0, actual = countReported(directory, "Serves players in the U.S. and Canada."))
        assertEquals(expected = 0, actual = countReported(directory, "Serves players in the U.S."))
        assertEquals(expected = 0, actual = countReported(directory, "Serves players in the U.S. Frees them."))
        assertEquals(expected = 1, actual = countReported(directory, "Serves players in the US. Frees them."))
    }

    @Test
    fun GIVEN_abbreviation_glued_to_a_word_WHEN_checked_THEN_it_is_not_an_abbreviation(@TempDir directory: File) {
        assertEquals(expected = 1, actual = countReported(directory, "Хранит рейтингиetc. for players."))
        assertEquals(expected = 1, actual = countReported(directory, "Хранит рейтингиetc. The next."))
        assertEquals(expected = 1, actual = countReported(directory, "Stores top10etc. for players."))
        assertEquals(expected = 1, actual = countReported(directory, "Stores top_etc. for players."))
        assertEquals(expected = 1, actual = countReported(directory, "Stores top²etc. for players."))
        assertEquals(expected = 1, actual = countReported(directory, "Stores topⅧetc. for players."))
        assertEquals(expected = 0, actual = countReported(directory, "Хранит рейтинги, etc. for players."))
    }

    @Test
    fun GIVEN_unit_without_readme_WHEN_checked_THEN_asks_for_one(@TempDir directory: File) {
        val violation = UnitReadme().check(unit(directory, isRoot = false)).single()

        assertTrue(violation.message.startsWith("Add a README.md of one sentence"))
    }

    @Test
    fun GIVEN_root_project_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        assertEquals(emptyList(), UnitReadme().check(unit(directory, isRoot = true)))
    }
}
