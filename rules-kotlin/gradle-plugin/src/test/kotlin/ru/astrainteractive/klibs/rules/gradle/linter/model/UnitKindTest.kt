@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.model

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class UnitKindTest {
    @Test
    fun GIVEN_every_kind_WHEN_looked_up_by_its_cli_name_THEN_returns_that_kind() {
        UnitKind.entries.forEach { kind ->
            assertEquals(kind, UnitKind.fromCliName(kind.cliName))
        }
    }

    @Test
    fun GIVEN_name_of_no_kind_WHEN_looked_up_THEN_returns_null() {
        assertNull(UnitKind.fromCliName("service"))
    }
}
