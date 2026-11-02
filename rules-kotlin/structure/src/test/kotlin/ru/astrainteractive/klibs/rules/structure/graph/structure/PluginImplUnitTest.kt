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

    @Test
    fun GIVEN_plugin_with_explicit_ui_and_mixed_core_WHEN_checked_THEN_reports_both_roles() {
        val plugin = units(
            ProductKind.PLATFORM_PLUGIN,
            ":feature:ui",
            ":core:common"
        )
        val ui = plugin[0].context!!.copy(kind = UnitKind.IMPL_UI)
        val core = plugin[1].context!!.copy(kind = UnitKind.CORE)
        val units = listOf(
            BuildUnit(
                ":feature:ui",
                File("/project"),
                false,
                false,
                emptySet(),
                context = ui
            ),
            BuildUnit(
                ":core:common",
                File("/project"),
                false,
                false,
                emptySet(),
                context = core
            )
        )

        assertEquals(
            listOf(":feature:ui", ":core:common"),
            PluginImplUnit().check(units).map(StructureViolation::subject)
        )
    }

    @Test
    fun GIVEN_ui_role_without_product_context_WHEN_checked_THEN_requests_product_kind() {
        val context = UnitContext(
            UnitKind.IMPL_UI,
            null,
            false,
            emptySet(),
            emptyList()
        )
        val ui = BuildUnit(
            ":feature:ui",
            File("/project"),
            false,
            false,
            emptySet(),
            context = context
        )

        assertFailsWith<IllegalArgumentException> { PluginImplUnit().check(listOf(ui)) }
    }

    @Test
    fun GIVEN_missing_context_or_published_impl_WHEN_checked_THEN_preserves_legacy_exemptions() {
        val contextless = BuildUnit(
            ":feature:impl",
            File("/project"),
            false,
            false,
            emptySet()
        )
        val published = BuildUnit(
            ":feature:impl",
            File("/project"),
            false,
            false,
            emptySet(),
            context = UnitContext(
                UnitKind.IMPL,
                ProductKind.PLATFORM_PLUGIN,
                true,
                emptySet(),
                emptyList()
            )
        )

        assertEquals(emptyList(), PluginImplUnit().check(listOf(contextless, published)))
    }
}
