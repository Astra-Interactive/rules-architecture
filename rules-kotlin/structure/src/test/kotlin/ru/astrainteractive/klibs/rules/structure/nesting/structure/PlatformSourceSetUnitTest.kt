@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.nesting.structure

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import java.io.File
import kotlin.test.assertEquals

internal class PlatformSourceSetUnitTest {
    private fun unit(path: String, isEntryPoint: Boolean = false): BuildUnit {
        return BuildUnit(
            path = path,
            directory = File("/project"),
            isEntryPoint = isEntryPoint,
            isRoot = false,
            projectDependencies = emptySet()
        )
    }

    @Test
    fun GIVEN_units_named_after_targets_WHEN_checked_THEN_reports_each_but_entry_points() {
        val units = listOf(
            unit(":components:shop:payment:android"),
            unit(":modules:core:ios"),
            unit(":instances:jvm", isEntryPoint = true),
            unit(":modules:gui:bukkit"),
            unit(":androidApp")
        )

        assertEquals(
            listOf(":components:shop:payment:android", ":modules:core:ios"),
            units.flatMap { unit -> PlatformSourceSetUnit().check(unit) }.map(StructureViolation::subject)
        )
    }
}
