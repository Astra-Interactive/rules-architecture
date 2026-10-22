@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.graph

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.StructureViolation
import java.io.File
import kotlin.test.assertEquals

internal class ApiUnitDependencyTest {

    private fun unit(path: String): BuildUnit {
        return BuildUnit(
            path = path,
            directory = File("/project"),
            isEntryPoint = false,
            isRoot = false,
            projectDependencies = setOf(
                ":modules:jail:api",
                ":modules:core:bukkit",
                ":modules:data:dao",
                ":modules:dictionary:contract",
                ":modules:economy:impl",
                ":modules:command",
                ":modules:gui:bukkit"
            )
        )
    }

    @Test
    fun GIVEN_api_unit_depending_on_feature_units_WHEN_checked_THEN_reports_each() {
        val violations = ApiUnitDependency().check(unit(":modules:gui:api"))

        assertEquals(
            listOf(":modules:command", ":modules:gui:bukkit"),
            violations.map(StructureViolation::subject)
        )
        assertEquals(
            "An api unit depends only on other api units, core and the storage contracts unit: drop " +
                ":modules:command or depend on its api unit.",
            violations.first().message
        )
    }

    @Test
    fun GIVEN_unit_that_is_not_api_WHEN_checked_THEN_reports_nothing() {
        assertEquals(emptyList(), ApiUnitDependency().check(unit(":modules:gui:bukkit")))
    }
}
