@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.util

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.gradle.fixture.IsolatedProjectsBuild
import java.io.File

internal class ProjectExtTest {
    private fun unitKindScript(plugins: String): String {
        return """
            import ru.astrainteractive.klibs.rules.gradle.linter.extension.model.LinterExtension

            plugins {
                $plugins
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }

            val linterUnitKind = extensions.getByType<LinterExtension>().unitKind
            val kindReport = layout.buildDirectory.file("unit-kind.txt")
            tasks.register("reportUnitKind") {
                val unitKind = linterUnitKind
                val reportFile = kindReport
                outputs.file(reportFile)
                doLast { reportFile.get().asFile.writeText(unitKind.get()) }
            }
        """.trimIndent()
    }

    @Test
    fun GIVEN_units_named_and_built_by_convention_WHEN_configured_THEN_detects_their_kind(
        @TempDir rootDirectory: File
    ) {
        val build = IsolatedProjectsBuild(rootDirectory)
            .withRootBuildScript("")
            .withModule(name = "api", buildScript = unitKindScript("`java-library`"))
            .withModule(name = "contract", buildScript = unitKindScript("`java-library`"))
            .withModule(name = "server", buildScript = unitKindScript("application"))
            .withModule(name = "exposed", buildScript = unitKindScript("`java-library`"))
            .withModule(name = "memory", buildScript = unitKindScript("`java-library`"))
            .withModule(name = "jail", buildScript = unitKindScript("`java-library`"))
            .withModule(name = "gui:bukkit", buildScript = unitKindScript("`java-library`"))
            .withModule(name = "core:bukkit", buildScript = unitKindScript("`java-library`"))
            .withModule(name = "instances:bukkit", buildScript = unitKindScript("`java-library`"))
        val modules = listOf(
            "api",
            "contract",
            "server",
            "exposed",
            "memory",
            "jail",
            "gui:bukkit",
            "core:bukkit",
            "instances:bukkit"
        )

        build.run(*modules.map { module -> ":$module:reportUnitKind" }.toTypedArray())

        val kinds = modules.map { module ->
            build.file("${module.replace(':', '/')}/build/unit-kind.txt").readText()
        }
        assertEquals(
            listOf(
                "api",
                "contract",
                "entry-point",
                "technology",
                "technology",
                "context",
                "technology",
                "context",
                "entry-point"
            ),
            kinds
        )
    }
}
