@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.task.gradle

import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.gradle.fixture.IsolatedProjectsBuild
import java.io.File

class ArchitectureGraphTaskTest {
    @Test
    fun GIVEN_acyclic_unit_graph_WHEN_graph_checked_THEN_passes(@TempDir rootDirectory: File) {
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript(JAVA_MODULE_SCRIPT)
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

    private companion object {
        val JAVA_MODULE_SCRIPT = """
            plugins {
                `java-library`
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }
        """.trimIndent()
    }
}
