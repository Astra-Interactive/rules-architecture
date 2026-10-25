@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.graph.structure

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import java.io.File
import kotlin.test.assertEquals

internal class UnitDependencyTest {

    private fun unit(path: String, isEntryPoint: Boolean, dependencies: Set<String>): BuildUnit {
        return BuildUnit(
            path = path,
            directory = File("/project"),
            isEntryPoint = isEntryPoint,
            isRoot = false,
            projectDependencies = dependencies
        )
    }

    @Test
    fun GIVEN_api_unit_depending_on_impl_WHEN_checked_THEN_reports_it() {
        val api = unit(
            ":jail:api",
            isEntryPoint = false,
            dependencies = setOf(
                ":economy:impl",
                ":economy:api",
                ":core"
            )
        )

        val violations = UnitDependency().check(api)

        assertEquals(listOf(":economy:impl"), violations.map(StructureViolation::subject))
        assertEquals(
            "An api unit never depends on an impl unit: depend on the api unit next to :economy:impl.",
            violations.single().message
        )
    }

    @Test
    fun GIVEN_context_depending_on_impl_WHEN_checked_THEN_reports_it() {
        val context = unit(
            ":jail:impl",
            isEntryPoint = false,
            dependencies = setOf(":economy:impl", ":economy:api")
        )

        assertEquals(
            listOf("Only the entry point depends on :economy:impl: depend on the api unit next to it."),
            UnitDependency().check(context).map(StructureViolation::message)
        )
    }

    @Test
    fun GIVEN_entry_point_depending_on_impl_units_WHEN_checked_THEN_reports_nothing() {
        val entryPoint = unit(
            ":instances:bukkit",
            isEntryPoint = true,
            dependencies = setOf(":jail:impl", ":core")
        )

        assertEquals(emptyList(), UnitDependency().check(entryPoint))
    }

    @Test
    fun GIVEN_contract_unit_with_project_dependencies_WHEN_checked_THEN_reports_each() {
        val contract = unit(
            ":dictionary:contract",
            isEntryPoint = false,
            dependencies = setOf(":core", ":kernel")
        )

        assertEquals(
            listOf(":core", ":kernel"),
            UnitDependency().check(contract).map(StructureViolation::subject)
        )
    }

    @Test
    fun GIVEN_units_depending_on_platform_units_WHEN_checked_THEN_only_core_platform_is_allowed() {
        val gui = unit(
            ":modules:gui:bukkit",
            isEntryPoint = false,
            dependencies = setOf(
                ":modules:core:bukkit",
                ":modules:gui:api",
                ":modules:event-bukkit",
                ":modules:menu:bukkit"
            )
        )

        assertEquals(
            listOf(":modules:event-bukkit", ":modules:menu:bukkit"),
            UnitDependency().check(gui).map(StructureViolation::subject)
        )
    }

    @Test
    fun GIVEN_core_unit_depending_on_a_feature_WHEN_checked_THEN_reports_it() {
        val core = unit(
            ":modules:core:bukkit",
            isEntryPoint = false,
            dependencies = setOf(":modules:core:api", ":modules:gui:api")
        )

        assertEquals(
            listOf("core depends on no feature unit: drop :modules:gui:api."),
            UnitDependency().check(core).map(StructureViolation::message)
        )
    }
}
