@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.graph

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.StructureViolation
import java.io.File
import kotlin.test.assertEquals

internal class TechnologyDependencyTest {

    private fun unit(path: String, isEntryPoint: Boolean): BuildUnit {
        return BuildUnit(
            path = path,
            directory = File("/project"),
            isEntryPoint = isEntryPoint,
            isRoot = false,
            projectDependencies = setOf(
                ":modules:data:exposed",
                ":modules:data:dao",
                ":core:cache4k"
            )
        )
    }

    @Test
    fun GIVEN_feature_unit_depending_on_technology_units_WHEN_checked_THEN_reports_each() {
        val violations = TechnologyDependency().check(unit(":modules:gui:api", isEntryPoint = false))

        assertEquals(
            listOf(":core:cache4k", ":modules:data:exposed"),
            violations.map(StructureViolation::subject)
        )
        assertEquals(
            "Depend on the contracts unit instead of the technology unit :core:cache4k: only the technology unit " +
                "and the entry point depend on the technology.",
            violations.first().message
        )
    }

    @Test
    fun GIVEN_entry_point_or_technology_unit_WHEN_checked_THEN_reports_nothing() {
        val entryPoint = TechnologyDependency().check(unit(":instances:bukkit", isEntryPoint = true))
        val technology = TechnologyDependency().check(unit(":modules:data:room", isEntryPoint = false))

        assertEquals(emptyList(), entryPoint + technology)
    }
}
