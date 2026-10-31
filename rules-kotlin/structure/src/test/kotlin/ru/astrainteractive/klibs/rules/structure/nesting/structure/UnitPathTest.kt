@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.nesting.structure

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import java.io.File
import kotlin.test.assertEquals

internal class UnitPathTest {
    private fun unit(path: String, isEntryPoint: Boolean = false, isLibrary: Boolean = false): BuildUnit {
        return BuildUnit(
            path = path,
            directory = File("/project"),
            isEntryPoint = isEntryPoint,
            isRoot = path == ":",
            projectDependencies = emptySet(),
            isLibrary = isLibrary
        )
    }

    @Test
    fun GIVEN_sub_context_unit_at_the_root_WHEN_checked_THEN_reports_it() {
        assertEquals(
            listOf(
                "Move :jail into a container, modules/jail: a sub-context unit lives under modules/<sub-context> or " +
                    "modules/<context>/<sub-context>.",
            ),
            UnitPath().check(unit(":jail")).map(StructureViolation::message)
        )
    }

    @Test
    fun GIVEN_units_in_containers_entry_points_libraries_and_root_WHEN_checked_THEN_reports_nothing() {
        val units = listOf(
            unit(":modules:jail"),
            unit(":components:shop:payment:impl"),
            unit(":androidApp", isEntryPoint = true),
            unit(":detekt", isLibrary = true),
            unit(":")
        )

        assertEquals(emptyList(), units.flatMap { unit -> UnitPath().check(unit) })
    }
}
