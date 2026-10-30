@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.declaration.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import dev.detekt.test.utils.compileContentForTest
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class FileLengthTest {
    private val rule = FileLength(Config.empty)

    private fun codeOfLines(count: Int): String {
        return List(count) { index -> "val value$index = $index" }.joinToString(separator = "\n")
    }

    @Test
    fun GIVEN_file_of_401_lines_WHEN_linted_THEN_reports_it() {
        val file = compileContentForTest(codeOfLines(401), filename = "Values.kt")

        assertEquals(
            listOf("Values.kt has 401 lines: split it, a file passes 400 lines only by mistake."),
            rule.lint(file).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_file_of_400_lines_WHEN_linted_THEN_reports_nothing() {
        assertEquals(emptyList(), rule.lint(codeOfLines(400)).map(Finding::message))
    }
}
