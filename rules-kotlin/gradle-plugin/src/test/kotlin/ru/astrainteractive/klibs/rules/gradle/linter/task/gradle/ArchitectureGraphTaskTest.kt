@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.task.gradle

import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.CheckPolicy
import ru.astrainteractive.klibs.rules.core.unit.model.ImplementationAccess
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UiComposition
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitPolicy
import ru.astrainteractive.klibs.rules.gradle.fixture.IsolatedProjectsBuild
import ru.astrainteractive.klibs.rules.gradle.linter.util.findGraphViolations
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import java.io.File
import kotlin.test.assertEquals

class ArchitectureGraphTaskTest {
    private fun graphUnit(
        path: String,
        kind: UnitKind,
        dependencies: Set<String> = emptySet(),
        disabled: Map<String, String> = emptyMap(),
        product: ProductKind = ProductKind.APP
    ): BuildUnit {
        val context = UnitContext(
            kind,
            product,
            false,
            emptySet(),
            emptyList(),
            UnitPolicy(
                UiComposition.LOCAL,
                ImplementationAccess.NONE,
                CheckPolicy(disabled, emptyMap())
            )
        )
        return BuildUnit(
            path,
            File("/project"),
            kind == UnitKind.ENTRY_POINT,
            path == ":",
            dependencies,
            context = context
        )
    }

    @Test
    fun GIVEN_owner_disables_boundary_WHEN_graph_checked_THEN_other_owner_and_cycle_remain() {
        val root = graphUnit(":", UnitKind.CONTEXT)
        val a = graphUnit(
            ":a:impl",
            UnitKind.IMPL,
            setOf(":b:impl"),
            mapOf("UnitDependency" to "temporary migration", "UnitDependencyCycle" to "local request")
        )
        val b = graphUnit(
            ":b:impl",
            UnitKind.IMPL,
            setOf(":a:impl")
        )

        val violations = listOf(
            root,
            a,
            b
        ).findGraphViolations()
        val relevantRules = setOf("UnitDependencyCycle", "UnitDependency")

        assertEquals(
            listOf(":a:impl -> :b:impl -> :a:impl", ":a:impl"),
            violations.filter { violation -> violation.rule in relevantRules }
                .map { violation -> violation.subject }
        )
    }

    @Test
    fun GIVEN_root_disables_cycle_WHEN_graph_checked_THEN_boundary_still_reports() {
        val root = graphUnit(
            ":",
            UnitKind.CONTEXT,
            disabled = mapOf("UnitDependencyCycle" to "investigating graph migration")
        )
        val a = graphUnit(
            ":a:impl",
            UnitKind.IMPL,
            setOf(":b:impl")
        )
        val b = graphUnit(
            ":b:impl",
            UnitKind.IMPL,
            setOf(":a:impl")
        )

        val violations = listOf(
            root,
            a,
            b
        ).findGraphViolations()

        assertEquals(emptyList(), violations.filter { violation -> violation.rule == "UnitDependencyCycle" })
        assertEquals(2, violations.count { violation -> violation.rule == "UnitDependency" })
    }

    @Test
    fun GIVEN_one_owner_disables_entrypoint_graph_check_WHEN_checked_THEN_other_owner_still_reports() {
        val root = graphUnit(":", UnitKind.CONTEXT)
        val app = graphUnit(":app", UnitKind.ENTRY_POINT)
        val quiet = graphUnit(
            ":quiet:api",
            UnitKind.API,
            setOf(":app"),
            mapOf("EntryPointGraphDependency" to "legacy startup dependency")
        )
        val active = graphUnit(
            ":active:api",
            UnitKind.API,
            setOf(":app")
        )

        val subjects = listOf(
            root,
            app,
            quiet,
            active
        ).findGraphViolations()
            .filter { violation -> violation.rule == "EntryPointGraphDependency" }
            .map { violation -> violation.path }

        assertEquals(listOf(":active:api"), subjects)
    }

    @Test
    fun GIVEN_one_plugin_unit_disables_role_check_WHEN_checked_THEN_other_unit_still_reports() {
        val root = graphUnit(":", UnitKind.CONTEXT)
        val quiet = graphUnit(
            ":quiet:impl",
            UnitKind.IMPL,
            disabled = mapOf("PluginImplUnit" to "legacy split"),
            product = ProductKind.PLATFORM_PLUGIN
        )
        val active = graphUnit(
            ":active:impl",
            UnitKind.IMPL,
            product = ProductKind.PLATFORM_PLUGIN
        )

        val subjects = listOf(
            root,
            quiet,
            active
        ).findGraphViolations()
            .filter { violation -> violation.rule == "PluginImplUnit" }
            .map { violation -> violation.subject }

        assertEquals(listOf(":active:impl"), subjects)
    }

    @Test
    fun GIVEN_acyclic_unit_graph_WHEN_graph_checked_THEN_passes(@TempDir rootDirectory: File) {
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript(
                """
                import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind

                $JAVA_MODULE_SCRIPT

                linter { productKind(ProductKind.APP) }
                """.trimIndent()
            )
            .withModule(name = "api", buildScript = JAVA_MODULE_SCRIPT)
            .withModule(name = "impl", buildScript = JAVA_MODULE_SCRIPT)

        build.run(":architectureGraphCheck")
    }

    @Test
    fun GIVEN_unit_depending_on_an_application_WHEN_graph_checked_THEN_reports_the_dependency(
        @TempDir rootDirectory: File
    ) {
        val appScript = """
            plugins {
                `java-library`
                application
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }
        """.trimIndent()
        val featureScript = """
            $JAVA_MODULE_SCRIPT

            dependencies { implementation(project(":app")) }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript(JAVA_MODULE_SCRIPT)
            .withModule(name = "app", buildScript = appScript)
            .withModule(name = "feature", buildScript = featureScript)

        val result = build.runAndFail(":architectureGraphCheck")

        assertTrue(
            result.output.contains("Drop the dependency of :feature on the entry point :app"),
            result.output
        )
    }

    @Test
    fun GIVEN_unit_dependency_cycle_WHEN_graph_checked_THEN_reports_the_cycle(@TempDir rootDirectory: File) {
        val aScript = """
            $JAVA_MODULE_SCRIPT

            dependencies { implementation(project(":b")) }
        """.trimIndent()
        val bScript = """
            $JAVA_MODULE_SCRIPT

            dependencies { implementation(project(":a")) }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript(JAVA_MODULE_SCRIPT)
            .withModule(name = "a", buildScript = aScript)
            .withModule(name = "b", buildScript = bScript)

        val result = build.runAndFail(":architectureGraphCheck")

        assertTrue(result.output.contains(":a -> :b -> :a"), result.output)
    }

    @Test
    fun GIVEN_entrypoint_depending_on_a_custom_named_fake_unit_WHEN_checked_THEN_rejects_the_production_edge(
        @TempDir rootDirectory: File
    ) {
        val fake = """
            import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind

            $JAVA_MODULE_SCRIPT

            linter { unitKind(UnitKind.FAKE) }
        """.trimIndent()
        val app = """
            $JAVA_MODULE_SCRIPT

            apply(plugin = "application")
            dependencies { implementation(project(":feature:helpers")) }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript(JAVA_MODULE_SCRIPT)
            .withModule(name = "app", buildScript = app)
            .withModule(name = "feature:helpers", buildScript = fake)

        val result = build.runAndFail(":architectureGraphCheck")

        assertTrue(result.output.contains("shared fakes are consumed only by tests"), result.output)
    }

    private companion object {
        val JAVA_MODULE_SCRIPT = """
            plugins {
                `java-library`
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }
        """.trimIndent()
    }
}
