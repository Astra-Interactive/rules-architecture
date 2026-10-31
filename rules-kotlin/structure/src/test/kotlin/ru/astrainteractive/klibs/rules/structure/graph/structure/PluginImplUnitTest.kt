@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.graph.structure

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import java.io.File
import kotlin.test.assertEquals

internal class PluginImplUnitTest {
    private fun units(vararg paths: String): List<BuildUnit> {
        return paths.map { path ->
            BuildUnit(
                path = path,
                directory = File("/project"),
                isEntryPoint = false,
                isRoot = false,
                projectDependencies = emptySet()
            )
        }
    }

    @Test
    fun GIVEN_plugin_with_an_impl_unit_WHEN_checked_THEN_reports_it() {
        val plugin = units(
            ":modules:gui:api",
            ":modules:gui:bukkit",
            ":modules:jail:impl"
        )

        assertEquals(listOf(":modules:jail:impl"), PluginImplUnit().check(plugin).map(StructureViolation::subject))
    }

    @Test
    fun GIVEN_app_with_impl_units_WHEN_checked_THEN_reports_nothing() {
        val app = units(
            ":components:shop:payment:api",
            ":components:shop:payment:impl",
            ":androidApp"
        )

        assertEquals(emptyList(), PluginImplUnit().check(app))
    }
}
