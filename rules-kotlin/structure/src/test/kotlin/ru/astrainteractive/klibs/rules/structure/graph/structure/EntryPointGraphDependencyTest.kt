@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.graph.structure

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import java.io.File
import kotlin.test.assertEquals

internal class EntryPointGraphDependencyTest {
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
    fun GIVEN_units_depending_on_an_application_WHEN_checked_THEN_reports_each_dependency() {
        val units = listOf(
            unit(
                ":androidApp",
                isEntryPoint = true,
                dependencies = setOf(":feature:impl")
            ),
            unit(
                ":feature:impl",
                isEntryPoint = false,
                dependencies = setOf(":androidApp", ":feature:api")
            ),
            unit(
                ":feature:api",
                isEntryPoint = false,
                dependencies = emptySet()
            )
        )

        assertEquals(
            listOf(":feature:impl -> :androidApp"),
            EntryPointGraphDependency().check(units).map { violation -> "${violation.path} -> ${violation.subject}" }
        )
    }

    @Test
    fun GIVEN_graph_without_dependencies_on_entry_points_WHEN_checked_THEN_reports_nothing() {
        val units = listOf(
            unit(
                ":androidApp",
                isEntryPoint = true,
                dependencies = setOf(":feature:impl")
            ),
            unit(
                ":feature:impl",
                isEntryPoint = false,
                dependencies = setOf(":feature:api")
            )
        )

        assertEquals(emptyList<StructureViolation>(), EntryPointGraphDependency().check(units))
    }
}
