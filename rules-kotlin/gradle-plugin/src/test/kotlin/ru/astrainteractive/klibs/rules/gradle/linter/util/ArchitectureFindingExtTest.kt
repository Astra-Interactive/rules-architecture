@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.util

import ru.astrainteractive.klibs.rules.gradle.linter.model.ArchitectureFinding
import kotlin.test.Test
import kotlin.test.assertEquals

class ArchitectureFindingExtTest {

    @Test
    fun GIVEN_finding_WHEN_described_THEN_names_path_message_and_rule() {
        val finding = ArchitectureFinding(
            rule = "KindShape",
            path = "src/main/kotlin/com/example/Jail.kt",
            subject = "Jail",
            message = "Move Jail into a kind package"
        )

        assertEquals(
            "src/main/kotlin/com/example/Jail.kt: Move Jail into a kind package [KindShape]",
            finding.describe()
        )
    }

    @Test
    fun GIVEN_finding_WHEN_written_as_a_line_THEN_parses_back_to_it() {
        val finding = ArchitectureFinding(
            rule = "ApiConfiguration",
            path = "build.gradle.kts",
            subject = "api: com.example:core",
            message = "Declare com.example:core with implementation"
        )

        assertEquals(finding, ArchitectureFinding.parse(finding.toLine()))
    }
}
