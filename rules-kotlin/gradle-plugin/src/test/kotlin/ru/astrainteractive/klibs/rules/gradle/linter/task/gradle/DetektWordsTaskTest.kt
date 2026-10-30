@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.task.gradle

import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.gradle.fixture.IsolatedProjectsBuild
import java.io.File

class DetektWordsTaskTest {
    @Test
    fun GIVEN_project_word_and_library_WHEN_words_written_THEN_every_technology_rule_gets_both(
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
            .withModule(name = "jail", buildScript = moduleScript)

        build.run(":jail:architectureDetektWords")

        assertEquals(
            listOf(
                "architecture:",
                "  BareIdentifier:",
                "    technologyWords: [ 'mongo', 'mongodb' ]",
                "  KindNameInPackage:",
                "    technologyWords: [ 'mongo', 'mongodb' ]",
                "  RunBlockingOutsideBoundary:",
                "    technologyWords: [ 'mongo', 'mongodb' ]",
            ),
            build.file("jail/build/detekt/architecture-words.yml").readLines()
        )
    }
}
