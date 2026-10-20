@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.model

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class ArchitectureFindingTest {
    private val finding = ArchitectureFinding(
        rule = "KindShape",
        path = "src/main/kotlin/com/example/Jail.kt",
        subject = "Jail",
        message = "Move Jail into a kind package"
    )

    private fun List<String>.joinToLine(): String = joinToString(separator = ArchitectureFinding.FIELD_SEPARATOR)

    @Test
    fun GIVEN_report_line_WHEN_parsed_THEN_splits_it_into_the_four_fields() {
        val line = listOf(finding.rule, finding.path, finding.subject, finding.message).joinToLine()

        assertEquals(finding, ArchitectureFinding.parse(line))
    }

    @Test
    fun GIVEN_message_with_a_tab_WHEN_parsed_THEN_keeps_the_tab_in_the_message() {
        val message = listOf("Move", "Jail").joinToLine()
        val line = listOf("KindShape", "Jail.kt", "Jail", message).joinToLine()

        assertEquals(message, ArchitectureFinding.parse(line).message)
    }

    @Test
    fun GIVEN_line_with_missing_fields_WHEN_parsed_THEN_fails_naming_the_line() {
        val line = listOf("KindShape", "Jail.kt").joinToLine()

        val error = assertFailsWith<IllegalArgumentException> { ArchitectureFinding.parse(line) }

        assertEquals("Malformed architecture report line: '$line'", error.message)
    }

    @Test
    fun GIVEN_findings_with_different_wording_WHEN_baseline_keys_compared_THEN_they_are_equal() {
        val reworded = finding.copy(message = "Put Jail into a kind package")

        assertEquals(finding.baselineKey, reworded.baselineKey)
        assertEquals(
            listOf("KindShape", "src/main/kotlin/com/example/Jail.kt", "Jail").joinToLine(),
            finding.baselineKey
        )
    }

    @Test
    fun GIVEN_finding_WHEN_described_THEN_names_path_message_and_rule() {
        assertEquals(
            "src/main/kotlin/com/example/Jail.kt: Move Jail into a kind package [KindShape]",
            finding.describe()
        )
    }
}
