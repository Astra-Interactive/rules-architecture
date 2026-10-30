@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.test.lint
import dev.detekt.test.utils.compileForTest
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.io.File
import kotlin.test.assertEquals

internal class RunBlockingOutsideBoundaryTest {
    private val rule = RunBlockingOutsideBoundary(Config.empty)

    private fun lintAt(file: File, code: String): Int {
        file.parentFile.mkdirs()
        file.writeText(code.trimIndent())
        return rule.lint(compileForTest(file.toPath())).size
    }

    @Test
    fun GIVEN_run_blocking_in_domain_code_WHEN_linted_THEN_reports_each_call(@TempDir directory: File) {
        val code = """
            package x.jail.usecase

            class FreeInmateUseCase {
                fun invoke() = runBlocking { }

                fun main() {
                    runBlocking { }
                }
            }
        """

        assertEquals(2, lintAt(File(directory, "src/main/kotlin/x/jail/usecase/FreeInmateUseCase.kt"), code))
    }

    @Test
    fun GIVEN_run_blocking_in_main_WHEN_linted_THEN_reports_nothing(@TempDir directory: File) {
        val code = """
            package x

            fun main() {
                fun start() = runBlocking { }
                runBlocking { start() }
            }
        """

        assertEquals(0, lintAt(File(directory, "src/main/kotlin/x/Main.kt"), code))
    }

    @Test
    fun GIVEN_run_blocking_in_technology_package_WHEN_linted_THEN_reports_nothing(@TempDir directory: File) {
        val code = """
            package x.data.exposed.rating

            internal class ExposedRatingRepository {
                fun load() = runBlocking { }
            }
        """
        val file = File(directory, "src/main/kotlin/x/data/exposed/rating/ExposedRatingRepository.kt")

        assertEquals(0, lintAt(file, code))
    }

    @Test
    fun GIVEN_run_blocking_in_test_sources_or_other_calls_WHEN_linted_THEN_reports_nothing(@TempDir directory: File) {
        val testCode = """
            package x.jail.usecase

            fun load() = runBlocking { }
        """
        val otherCall = """
            package x.jail.usecase

            fun load() = runCatching { }
        """

        assertEquals(0, lintAt(File(directory, "src/test/kotlin/x/jail/usecase/LoadTest.kt"), testCode))
        assertEquals(0, lintAt(File(directory, "src/main/kotlin/x/jail/usecase/Load.kt"), otherCall))
    }
}
