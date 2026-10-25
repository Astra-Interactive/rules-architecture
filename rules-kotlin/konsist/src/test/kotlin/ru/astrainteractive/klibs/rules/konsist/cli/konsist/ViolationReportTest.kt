@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.cli.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File
import kotlin.test.assertEquals

internal class ViolationReportTest {
    private fun tsvLine(vararg fields: String): String = fields.joinToString(separator = TAB.toString())

    @Test
    fun GIVEN_violations_with_tabs_and_line_breaks_WHEN_written_THEN_writes_one_line_each(@TempDir directory: File) {
        val reportFile = File(directory, "nested/report.tsv")

        writeReport(
            listOf(
                Violation(
                    "LooseRootFile",
                    "src/A.kt",
                    "Jail",
                    "Move${TAB}Jail${LINE_FEED}out"
                ),
                Violation(
                    "TestMirror",
                    "src/B.kt",
                    "BTest",
                    "Move BTest"
                ),
            ),
            reportFile
        )

        assertEquals(
            listOf(
                tsvLine(
                    "LooseRootFile",
                    "src/A.kt",
                    "Jail",
                    "Move Jail out"
                ),
                tsvLine(
                    "TestMirror",
                    "src/B.kt",
                    "BTest",
                    "Move BTest"
                ),
            ),
            reportFile.readLines()
        )
    }

    @Test
    fun GIVEN_no_violation_WHEN_written_THEN_report_is_empty(@TempDir directory: File) {
        val reportFile = File(directory, "report.tsv")

        writeReport(emptyList(), reportFile)

        assertEquals("", reportFile.readText())
    }

    private companion object {
        const val TAB = '\t'
        const val LINE_FEED = '\n'
    }
}
