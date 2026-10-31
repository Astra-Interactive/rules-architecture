@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.task.gradle

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
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

        assertEquals(
            listOf(
                "architecture:",
                "  BareIdentifier:",
                "    technologyWords: [ 'mongo', 'mongodb' ]",
                "  KindNameInPackage:",
                "    technologyWords: [ 'mongo', 'mongodb' ]",
                "  RunBlockingOutsideBoundary:",
                "    technologyWords: [ 'mongo', 'mongodb' ]",
                "  RunCatchingCancellation:",
                "    inlineCancellationCheck: true",
            ),
            build.file("modules/jail/build/detekt/architecture-settings.yml").readLines()
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
}
