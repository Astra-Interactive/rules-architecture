@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.plugin.gradle

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.util.toUnitContext
import ru.astrainteractive.klibs.rules.gradle.fixture.IsolatedProjectsBuild
import java.io.File
import kotlin.test.assertEquals
import kotlin.test.assertTrue

internal class UnitPolicyIntegrationTest {
    @Test
    fun GIVEN_detekt_finding_WHEN_role_disables_it_THEN_engine_stops_reporting_it(
        @TempDir directory: File
    ) {
        val module = MODULE_SCRIPT.replace("`java-library`", "id(\"org.jetbrains.kotlin.jvm\")")
        val build = IsolatedProjectsBuild(directory)
            .online()
            .withRootBuildScript(ROOT_SCRIPT)
            .withModule("modules:coordinates", module)
            .withSourceFile(
                "modules/coordinates/src/main/kotlin/com/example/coordinates/Coordinates.kt",
                """
                    package com.example.coordinates

                    internal fun createCoordinates(): Pair<Int, Int> = Pair(1, 2)

                """.trimIndent()
            )
        val active = build.runAndFail(":modules:coordinates:detektMain")
        assertTrue(active.output.contains("AnonymousTuple"), active.output)

        build.file("modules/coordinates/build.gradle.kts").writeText(
            module + """

                linter {
                    unit(UnitRole.Context(checks = Checks(
                        disabled = mapOf(RuleId.ANONYMOUS_TUPLE to "compatibility fixture")
                    )))
                }
            """.trimIndent()
        )
        build.run(":modules:coordinates:detektMain")
    }

    @Test
    fun GIVEN_typed_edge_exception_WHEN_all_boundary_checks_run_THEN_accepts_and_reports_it(
        @TempDir directory: File
    ) {
        val build = IsolatedProjectsBuild(directory)
            .online()
            .withTypeSafeAccessors()
            .withRootBuildScript(ROOT_SCRIPT)
            .withModule("modules:offer:impl", MODULE_SCRIPT + "\nlinter { unit(UnitRole.Impl()) }")
            .withModule(
                "modules:consumer",
                MODULE_SCRIPT + """

                    dependencies { implementation(projects.modules.offer.impl) }
                    linter {
                        unit(UnitRole.Api(checks = Checks(
                            disabled = mapOf(RuleId.UNIT_README to "fixture module"),
                            allowedDependencies = mapOf(projects.modules.offer.impl.toUnitReference() to "migration")
                        )))
                    }
                """.trimIndent()
            )

        build.run(":modules:consumer:architectureCheck", ":architectureGraphCheck")
        val repeated = build.run(":modules:consumer:architectureCheck", ":architectureGraphCheck")

        assertTrue(repeated.output.contains("Configuration cache entry reused"), repeated.output)
        assertEquals("", build.file("modules/consumer/build/reports/architecture/violations.txt").readText())
        val report = build.file("modules/consumer/build/reports/architecture/policy.txt").readText()
        assertTrue(report.contains("SKIPPED UnitReadme: fixture module"), report)
        assertTrue(report.contains("ALLOWED :modules:consumer -> :modules:offer:impl: migration"), report)
    }

    @Test
    fun GIVEN_sibling_ui_WHEN_target_role_changes_THEN_invalidates_cached_graph(
        @TempDir directory: File
    ) {
        val build = IsolatedProjectsBuild(directory)
            .withRootBuildScript(ROOT_SCRIPT)
            .withModule("modules:offer:impl", MODULE_SCRIPT + "\nlinter { unit(UnitRole.Impl()) }")
            .withModule(
                "modules:offer:ui",
                MODULE_SCRIPT + """

                    dependencies { implementation(project(":modules:offer:impl")) }
                    linter { unit(UnitRole.ImplUi()) }
                """.trimIndent()
            )
        build.run(":architectureGraphCheck", ":modules:offer:ui:architectureDetektSettings")
        val encoded = build.file("modules/offer/ui/build/detekt/unit-context.txt").readText()
        assertEquals(UnitKind.IMPL_UI, encoded.toUnitContext().kind)

        build.file("modules/offer/impl/build.gradle.kts")
            .writeText(MODULE_SCRIPT + "\nlinter { unit(UnitRole.Technology()) }")
        val changed = build.runAndFail(":architectureGraphCheck")

        assertTrue(changed.output.contains(":modules:offer:impl"), changed.output)
        assertTrue(changed.output.contains("not allowed"), changed.output)
    }

    private companion object {
        val ROOT_SCRIPT = """
            import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
            plugins { id("ru.astrainteractive.klibs.rules.gradle.linter") }
            linter { productKind(ProductKind.APP) }
        """.trimIndent()

        val MODULE_SCRIPT = """
            import ru.astrainteractive.klibs.rules.core.unit.model.Checks
            import ru.astrainteractive.klibs.rules.core.unit.model.RuleId
            import ru.astrainteractive.klibs.rules.core.unit.model.UnitRole
            import ru.astrainteractive.klibs.rules.gradle.linter.util.toUnitReference
            plugins {
                `java-library`
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }
        """.trimIndent()
    }
}
