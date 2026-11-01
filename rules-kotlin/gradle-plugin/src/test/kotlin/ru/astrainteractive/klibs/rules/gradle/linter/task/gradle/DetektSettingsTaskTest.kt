@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.task.gradle

import org.gradle.testkit.runner.TaskOutcome
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.util.toUnitContext
import ru.astrainteractive.klibs.rules.gradle.fixture.IsolatedProjectsBuild
import java.io.File

class DetektSettingsTaskTest {
    @Test
    fun GIVEN_project_word_and_library_WHEN_settings_written_THEN_every_technology_rule_gets_both(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            plugins {
                `java-library`
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }

            linter {
                technologyWords.add("mongo")
            }

            dependencies {
                implementation("org.mongodb:mongodb-driver-kotlin-coroutine:5.2.0")
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .withRootBuildScript("")
            .withModule(name = "modules:jail", buildScript = moduleScript)

        build.run(":modules:jail:architectureDetektSettings")

        val lines = build.file("modules/jail/build/detekt/architecture-settings.yml").readLines()
        val contexts = lines.filter { line -> line.trimStart().startsWith("unitContextFile:") }
            .map { line -> File(line.substringAfter("'").substringBeforeLast("'")).readText().toUnitContext() }
        assertEquals(4, contexts.size)
        contexts.forEach { context ->
            assertEquals(UnitKind.CONTEXT, context.kind)
            assertEquals(setOf("mongo", "mongodb"), context.technologyWords)
        }
        assertEquals(
            listOf(
                "architecture:",
                "  BareIdentifier:",
                "    technologyWords: [ 'mongo', 'mongodb' ]",
                "  KindNameInPackage:",
                "    technologyWords: [ 'mongo', 'mongodb' ]",
                "  RunBlockingOutsideBoundary:",
                "    technologyWords: [ 'mongo', 'mongodb' ]",
                "  FieldDependencyConstruction:",
                "  RunCatchingCancellation:",
                "    inlineCancellationCheck: true",
            ),
            lines.filterNot { line -> line.trimStart().startsWith("unitContextFile:") }
        )
    }

    @Test
    fun GIVEN_mikro_core_in_the_version_catalog_WHEN_settings_written_THEN_refuses_the_inline_check(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            plugins {
                `java-library`
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }
        """.trimIndent()
        val catalog = """
            [libraries]
            klibs-mikro-core = { module = "ru.astrainteractive.klibs:mikro-core", version = "1.0.0" }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .withRootBuildScript("")
            .withModule(name = "modules:jail", buildScript = moduleScript)
            .withSourceFile("gradle/libs.versions.toml", catalog)

        build.run(":modules:jail:architectureDetektSettings")

        assertEquals(
            "    inlineCancellationCheck: false",
            build.file("modules/jail/build/detekt/architecture-settings.yml").readLines().last()
        )
    }

    @Test
    fun GIVEN_context_larger_than_yaml_limit_WHEN_detekt_runs_THEN_reads_it_and_tracks_context_changes(
        @TempDir rootDirectory: File
    ) {
        val moduleScript = """
            plugins {
                id("org.jetbrains.kotlin.jvm")
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }

            kotlin.sourceSets.named("main") {
                kotlin.srcDirs((1..600).map { index -> "src/generated/compilation-" + index + "/kotlin" })
            }
        """.trimIndent() + "\n"
        val build = IsolatedProjectsBuild(File(rootDirectory, "project's sources"))
            .online()
            .withRootBuildScript("")
            .withModule(name = "modules:jail", buildScript = moduleScript)
            .withSourceFile(
                "modules/jail/src/main/kotlin/sample/Entry.kt",
                """
                    package sample

                    class Entry
                """.trimIndent() + "\n"
            )

        val first = build.run(":modules:jail:detekt")
        val contextFile = build.file("modules/jail/build/detekt/unit-context.txt")
        val settingsFile = build.file("modules/jail/build/detekt/architecture-settings.yml")
        val initialContext = contextFile.readText()
        val initialSettings = settingsFile.readText()
        assertTrue(initialContext.length > 102_400)
        assertTrue(initialSettings.length < 102_400)
        assertEquals(TaskOutcome.SUCCESS, first.task(":modules:jail:detekt")?.outcome)
        val unchanged = build.run(":modules:jail:detekt")
        assertEquals(
            TaskOutcome.UP_TO_DATE,
            unchanged.task(":modules:jail:architectureDetektSettings")?.outcome,
            unchanged.output
        )

        build.file("modules/jail/build.gradle.kts").writeText(moduleScript.replace("1..600", "1..601"))
        val changed = build.run(":modules:jail:detekt")

        assertEquals(initialSettings, settingsFile.readText())
        assertTrue(initialContext != contextFile.readText())
        assertEquals(TaskOutcome.SUCCESS, changed.task(":modules:jail:detekt")?.outcome)
    }
}
