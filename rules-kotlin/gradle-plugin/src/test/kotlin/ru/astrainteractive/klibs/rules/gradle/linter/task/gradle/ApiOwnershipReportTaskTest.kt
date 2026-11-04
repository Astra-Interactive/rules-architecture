@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.task.gradle

import org.gradle.testkit.runner.TaskOutcome
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.gradle.fixture.IsolatedProjectsBuild
import java.io.File
import kotlin.test.assertContains
import kotlin.test.assertEquals
import kotlin.test.assertFalse

class ApiOwnershipReportTaskTest {
    @Test
    fun GIVEN_sibling_not_configured_WHEN_report_runs_with_cod_THEN_fails_instead_of_reporting_root_only(
        @TempDir root: File
    ) {
        val script = """
            plugins {
                id("org.jetbrains.kotlin.jvm")
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(root).online()
            .withRootBuildScript("""plugins { id("ru.astrainteractive.klibs.rules.gradle.linter") }""")
            .withModule("voice:api", script)
            .withModule("voice:impl", script)
            .withSourceFile("voice/api/src/main/kotlin/Local.kt", "package x; class Local")
            .withSourceFile(
                "voice/impl/src/main/kotlin/Caller.kt",
                "package y; import x.Local; class Caller(val x: Local)"
            )
        val task = ":architectureApiOwnershipReport"

        val incomplete = build.runAndFail(task, "--configure-on-demand")
        assertContains(incomplete.output, "incomplete API ownership report")
        val report = build.file("build/reports/architecture/api-ownership.txt")
        assertFalse(report.exists())

        val complete = build.run(task, "--no-configure-on-demand")
        assertEquals(TaskOutcome.SUCCESS, complete.task(task)?.outcome)
        assertContains(report.readText(), "CANDIDATE x.Local")
    }

    @Test
    fun GIVEN_local_api_type_WHEN_report_runs_THEN_is_advisory_and_cache_tracks_consumers(@TempDir root: File) {
        val script = """
            plugins {
                id("org.jetbrains.kotlin.jvm")
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(root).online()
            .withRootBuildScript("""plugins { id("ru.astrainteractive.klibs.rules.gradle.linter") }""")
            .withModule("voice:api", script)
            .withModule(
                "voice:impl",
                """
                $script
                dependencies { implementation(project(":voice:api")) }
                """.trimIndent()
            )
            .withSourceFile("voice/api/src/main/kotlin/Local.kt", "package x; class Local")
            .withSourceFile(
                "voice/impl/src/main/kotlin/Caller.kt",
                "package y; import x.Local; class Caller(val x: Local)"
            )
        val task = ":architectureApiOwnershipReport"

        val first = build.run(task)
        val report = build.file("build/reports/architecture/api-ownership.txt")
        assertEquals(TaskOutcome.SUCCESS, first.task(task)?.outcome)
        assertContains(report.readText(), "CANDIDATE x.Local")
        val second = build.run(task)
        assertContains(second.output, "Reusing configuration cache")
        assertEquals(TaskOutcome.UP_TO_DATE, second.task(task)?.outcome)
        build.withSourceFile("voice/impl/src/main/kotlin/Caller.kt", "package y; class Caller")
        val third = build.run(task)
        assertEquals(TaskOutcome.SUCCESS, third.task(task)?.outcome)
        assertContains(report.readText(), "UNKNOWN x.Local")
        assertFalse(report.readText().contains("CANDIDATE x.Local"))
    }
}
