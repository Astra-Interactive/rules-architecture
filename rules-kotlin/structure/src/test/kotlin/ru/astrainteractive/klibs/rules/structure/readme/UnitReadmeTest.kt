@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.readme

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
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
    fun GIVEN_unit_without_readme_WHEN_checked_THEN_asks_for_one(@TempDir directory: File) {
        val violation = UnitReadme().check(unit(directory, isRoot = false)).single()

        assertTrue(violation.message.startsWith("Add a README.md of one sentence"))
    }

    @Test
    fun GIVEN_root_project_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        assertEquals(emptyList(), UnitReadme().check(unit(directory, isRoot = true)))
    }
}
