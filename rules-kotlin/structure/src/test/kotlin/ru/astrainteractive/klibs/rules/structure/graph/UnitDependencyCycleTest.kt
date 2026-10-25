@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.graph

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.StructureViolation
import java.io.File
import kotlin.test.assertEquals

internal class UnitDependencyCycleTest {
    private fun unit(path: String, vararg dependencies: String): BuildUnit {
        return BuildUnit(
            path = path,
            directory = File("/project"),
            isEntryPoint = false,
            isRoot = false,
            projectDependencies = dependencies.toSet()
        )
    }

    @Test
    fun GIVEN_acyclic_and_external_dependencies_WHEN_checked_THEN_reports_nothing() {
        val units = listOf(
            unit(
                ":entry",
                ":api",
                ":missing"
            ),
            unit(":api", ":core"),
            unit(":core")
        )

        assertEquals(emptyList(), UnitDependencyCycle().check(units))
    }

    @Test
    fun GIVEN_cycles_reachable_from_several_units_WHEN_checked_THEN_reports_each_cycle_once() {
        val units = listOf(
            unit(":a", ":b"),
            unit(":b", ":c"),
            unit(":c", ":a"),
            unit(":self", ":self"),
            unit(
                ":external",
                ":missing",
                ":a"
            )
        )

        val cycles = UnitDependencyCycle().check(units)

        assertEquals(
            listOf(":a -> :b -> :c -> :a", ":self -> :self"),
            cycles.map(StructureViolation::subject)
        )
        assertEquals("UnitDependencyCycle", cycles.first().rule)
        assertEquals(
            "Break the unit dependency cycle :a -> :b -> :c -> :a: the project units form an acyclic graph.",
            cycles.first().message
        )
    }
}
