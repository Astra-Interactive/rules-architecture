@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.graph.structure

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import java.io.File
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

internal class PluginImplUnitTest {
    private fun units(product: ProductKind?, vararg paths: String): List<BuildUnit> {
        return paths.map { path ->
            BuildUnit(
                path = path,
                directory = File("/project"),
                isEntryPoint = false,
                isRoot = false,
                projectDependencies = emptySet(),
                context = UnitContext(
                    if (path.endsWith(":impl")) UnitKind.IMPL else UnitKind.CONTEXT,
                    product,
                    false,
                    emptySet(),
                    emptyList()
                )
            )
        }
    }

    @Test
    fun GIVEN_plugin_with_an_impl_unit_WHEN_checked_THEN_reports_it() {
        val plugin = units(
            ProductKind.PLATFORM_PLUGIN,
            ":modules:gui:api",
            ":modules:gui:bukkit",
            ":modules:jail:impl"
        )

        assertEquals(listOf(":modules:jail:impl"), PluginImplUnit().check(plugin).map(StructureViolation::subject))
    }

    @Test
    fun GIVEN_app_with_impl_units_WHEN_checked_THEN_reports_nothing() {
        val app = units(
            ProductKind.APP,
            ":components:shop:payment:api",
            ":components:shop:payment:impl",
            ":components:shop:bukkit",
            ":androidApp"
        )

        assertEquals(emptyList(), PluginImplUnit().check(app))
    }

    @Test
    fun GIVEN_impl_without_product_context_WHEN_checked_THEN_requires_the_missing_enum() {
        assertFailsWith<IllegalArgumentException> {
            PluginImplUnit().check(units(null, ":feature:impl"))
        }
    }
}
