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

    private fun compilation(name: String, vararg dependencies: String, test: Boolean = false): UnitContext.Compilation {
        return UnitContext.Compilation(
            name,
            emptyMap(),
            test,
            null,
            dependencies.toSet()
        )
    }

    private fun compiledUnit(
        path: String,
        vararg compilations: UnitContext.Compilation,
        kind: UnitKind = UnitKind.API
    ): BuildUnit {
        val context = UnitContext(
            kind,
            ProductKind.APP,
            false,
            emptySet(),
            compilations.toList()
        )
        val dependencies = compilations.filterNot(UnitContext.Compilation::test)
            .flatMap { scope -> scope.projectDependencies }.toSet()
        return BuildUnit(
            path,
            File("/project"),
            false,
            false,
            dependencies,
            context = context
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

    @Test
    fun GIVEN_opposing_android_and_ios_edges_WHEN_checked_THEN_does_not_invent_a_cross_platform_cycle() {
        val units = listOf(
            compiledUnit(
                ":a",
                compilation("androidJvm:android:debug", ":b"),
                compilation("native-ios_arm64:iosArm64:main")
            ),
            compiledUnit(
                ":b",
                compilation("androidJvm:android:debug"),
                compilation("native-ios_arm64:iosArm64:main", ":a")
            )
        )

        assertEquals(emptyList(), UnitDependencyCycle().check(units))
    }

    @Test
    fun GIVEN_cycle_in_compatible_jvm_compilations_WHEN_checked_THEN_reports_it_despite_different_target_names() {
        val units = listOf(
            compiledUnit(":a", compilation("jvm:desktop:main", ":b")),
            compiledUnit(":b", compilation("jvm:server:main", ":a"))
        )

        assertEquals(listOf(":a -> :b -> :a"), UnitDependencyCycle().check(units).map(StructureViolation::subject))
    }

    @Test
    fun GIVEN_api_tests_using_its_fake_unit_WHEN_checked_THEN_does_not_turn_test_edges_into_production_cycles() {
        val units = listOf(
            compiledUnit(
                ":feature:api",
                compilation("jvm:jvm:main"),
                compilation(
                    "jvm:jvm:test",
                    ":feature:fake",
                    test = true
                )
            ),
            compiledUnit(":feature:fake", compilation("jvm:jvm:main", ":feature:api"))
        )

        assertEquals(emptyList(), UnitDependencyCycle().check(units))
    }

    @Test
    fun GIVEN_split_design_system_WHEN_checked_THEN_allows_acyclic_ui_dependencies_and_reports_a_real_cycle() {
        val settings = compiledUnit(
            ":services:settings",
            compilation("jvm:jvm:main"),
            kind = UnitKind.CORE_API
        )
        val theme = compiledUnit(
            ":design:tokens",
            compilation("jvm:jvm:main", ":services:settings"),
            kind = UnitKind.CORE_UI
        )
        val components = compiledUnit(
            ":design:components",
            compilation("jvm:jvm:main", ":design:tokens"),
            kind = UnitKind.CORE_UI
        )
        val sheet = compiledUnit(
            ":design:sheet",
            compilation(
                "jvm:jvm:main",
                ":design:components",
                ":design:tokens"
            ),
            kind = UnitKind.CORE_UI
        )

        assertEquals(
            emptyList(),
            UnitDependencyCycle().check(
                listOf(
                    settings,
                    theme,
                    components,
                    sheet
                )
            )
        )

        val cyclicTheme = compiledUnit(
            ":design:tokens",
            compilation("jvm:jvm:main", ":design:components"),
            kind = UnitKind.CORE_UI
        )
        assertEquals(
            listOf(":design:components -> :design:tokens -> :design:components"),
            UnitDependencyCycle().check(
                listOf(
                    settings,
                    cyclicTheme,
                    components,
                    sheet
                )
            )
                .map(StructureViolation::subject)
        )
    }

    @Test
    fun GIVEN_several_ambiguous_same_platform_targets_WHEN_checked_THEN_does_not_union_their_dependencies() {
        val units = listOf(
            compiledUnit(
                ":a",
                compilation("jvm:first:main", ":b"),
                compilation("jvm:second:main")
            ),
            compiledUnit(":b", compilation("jvm:first:main", ":a"))
        )

        assertEquals(emptyList(), UnitDependencyCycle().check(units))
    }
}
