@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.graph

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
import java.io.File
import kotlin.test.assertEquals

internal class UnitDependencyTest {

    private fun unit(path: String, isEntryPoint: Boolean, dependencies: Set<String>): BuildUnit = BuildUnit(
        path = path,
        directory = File("/project"),
        isEntryPoint = isEntryPoint,
        isRoot = false,
        projectDependencies = dependencies
    )

    @Test
    fun GIVEN_api_unit_depending_on_impl_WHEN_checked_THEN_reports_it() {
        val api = unit(
            ":jail:api",
            isEntryPoint = false,
            dependencies = setOf(":economy:impl", ":economy:api", ":core")
        )

        val violations = UnitDependency().check(api)

        assertEquals(listOf(":economy:impl"), violations.map { violation -> violation.subject })
        assertEquals(
            "An api unit never depends on an impl unit: depend on the api unit next to :economy:impl.",
            violations.single().message
        )
    }

    @Test
    fun GIVEN_context_depending_on_impl_WHEN_checked_THEN_reports_it() {
        val context = unit(":jail:impl", isEntryPoint = false, dependencies = setOf(":economy:impl", ":economy:api"))

        assertEquals(
            listOf("Only the entry point depends on :economy:impl: depend on the api unit next to it."),
            UnitDependency().check(context).map { violation -> violation.message }
        )
    }

    @Test
    fun GIVEN_entry_point_depending_on_impl_units_WHEN_checked_THEN_reports_nothing() {
        val entryPoint = unit(":instances:bukkit", isEntryPoint = true, dependencies = setOf(":jail:impl", ":core"))

        assertEquals(emptyList(), UnitDependency().check(entryPoint))
    }

    @Test
    fun GIVEN_contract_unit_with_project_dependencies_WHEN_checked_THEN_reports_each() {
        val contract = unit(":dictionary:contract", isEntryPoint = false, dependencies = setOf(":core", ":kernel"))

        assertEquals(
            listOf(":core", ":kernel"),
            UnitDependency().check(contract).map { violation -> violation.subject }
        )
    }
}
