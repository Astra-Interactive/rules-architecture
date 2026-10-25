@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.result.detekt

import dev.detekt.api.Config
import dev.detekt.test.lint
import dev.detekt.test.utils.compileForTest
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.io.File
import kotlin.test.assertEquals

internal class GetOrThrowInMainTest {
    private val rule = GetOrThrowInMain(Config.empty)

    private fun lintAt(directory: File, path: String): Int {
        val file = File(directory, path)
        file.parentFile.mkdirs()
        file.writeText(CODE)
        return rule.lint(compileForTest(file.toPath())).size
    }

    @Test
    fun GIVEN_get_or_throw_in_main_sources_WHEN_linted_THEN_reports_it(@TempDir directory: File) {
        assertEquals(1, lintAt(directory, "src/main/kotlin/Loader.kt"))
        assertEquals(1, lintAt(directory, "src/commonMain/kotlin/Loader.kt"))
    }

    @Test
    fun GIVEN_get_or_throw_in_test_sources_WHEN_linted_THEN_reports_nothing(@TempDir directory: File) {
        val findings = listOf(
            "src/test/kotlin/LoaderTest.kt",
            "src/commonTest/kotlin/LoaderTest.kt",
            "src/androidUnitTest/kotlin/LoaderTest.kt",
            "src/testFixtures/kotlin/FakeLoader.kt",
        ).sumOf { path -> lintAt(directory, path) }

        assertEquals(0, findings)
    }

    @Test
    fun GIVEN_get_or_throw_with_a_default_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun read(result: Result<Int>) = result.getOrThrow(1)
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    private companion object {
        val CODE = """
            fun read(result: Result<Int>): Int = result.getOrThrow()
        """.trimIndent()
    }
}
