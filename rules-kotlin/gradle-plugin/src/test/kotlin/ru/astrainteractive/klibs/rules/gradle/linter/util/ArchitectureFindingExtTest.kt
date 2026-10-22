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
}
