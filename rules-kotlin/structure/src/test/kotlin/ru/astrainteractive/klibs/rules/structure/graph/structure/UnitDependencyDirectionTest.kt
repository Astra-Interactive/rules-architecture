@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.graph.structure

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import java.io.File
import kotlin.test.assertEquals

internal class UnitDependencyDirectionTest {
    private fun forbidden(
        path: String,
        dependencies: Set<String>,
        isEntryPoint: Boolean = false,
        isLibrary: Boolean = false
    ): List<String> {
        val unit = BuildUnit(
            path = path,
            directory = File("/project"),
            isEntryPoint = isEntryPoint,
            isRoot = false,
            projectDependencies = dependencies,
            isLibrary = isLibrary
        )
        return UnitDependencyDirection.findForbidden(unit)
    }

    @Test
    fun GIVEN_documented_unit_dependencies_WHEN_classified_THEN_accepts_only_allowed_targets() {
        val allowed = mapOf(
            ":instances:bukkit" to ":feature:event-bukkit",
            ":gui:bukkit" to ":core:bukkit",
            ":event-bukkit" to ":data:contract",
            ":gui:api" to ":data:dao",
            ":jail:impl" to ":economy:api",
            ":data:exposed" to ":data:contract",
            ":data:dao" to ":core",
            ":core" to ":",
            ":core:bukkit" to ":core:api",
            ":core:ui" to ":core:api",
            ":payment:impl" to ":core:ui",
            ":dictionary:impl" to ":dictionary:contract",
            ":jail:fake" to ":jail:api"
        )

        allowed.forEach { (path, dependency) ->
            val dependencies = if (dependency == ":") emptySet() else setOf(dependency)
            assertEquals(
                emptyList(),
                forbidden(
                    path,
                    dependencies,
                    isEntryPoint = path == ":instances:bukkit"
                ),
                path
            )
        }
    }

    @Test
    fun GIVEN_dependencies_outside_the_direction_table_WHEN_classified_THEN_reports_each() {
        val forbidden = mapOf(
            ":gui:bukkit" to ":event-bukkit",
            ":event-bukkit" to ":gui:bukkit",
            ":gui:api" to ":core:bukkit",
            ":jail:impl" to ":event-bukkit",
            ":data:exposed" to ":other:room",
            ":data:dao" to ":jail:api",
            ":dictionary:contract" to ":core",
            ":core" to ":jail:api",
            ":core:bukkit" to ":jail:api",
            ":core:ui" to ":core:bukkit",
            ":payment:api" to ":core:ui",
            ":jail:fake" to ":economy:api"
        )

        forbidden.forEach { (path, dependency) ->
            assertEquals(
                listOf(dependency),
                forbidden(path, setOf(dependency)),
                path
            )
        }
    }

    @Test
    fun GIVEN_published_library_dependencies_WHEN_classified_THEN_accepts_them() {
        assertEquals(
            emptyList(),
            forbidden(
                ":gradle-plugin",
                setOf(":detekt", ":structure"),
                isLibrary = true
            )
        )
    }

    @Test
    fun GIVEN_core_unit_with_strict_explicit_api_WHEN_classified_THEN_still_depends_on_no_feature() {
        assertEquals(
            listOf(":gui:api"),
            forbidden(
                ":core:api",
                setOf(":gui:api"),
                isLibrary = true
            )
        )
    }
}
