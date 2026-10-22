@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.testing

import dev.detekt.api.Config
import dev.detekt.test.lint
import dev.detekt.test.utils.compileForTest
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.io.File
import kotlin.test.assertEquals

internal class RunBlockingInTestTest {
    private val rule = RunBlockingInTest(Config.empty)

    private fun lintAt(directory: File, path: String): List<String> {
        val file = File(directory, path)
        file.parentFile.mkdirs()
        file.writeText(CODE)
        return rule.lint(compileForTest(file.toPath())).map { finding -> finding.message }
    }

    @Test
    fun GIVEN_run_blocking_in_test_sources_WHEN_linted_THEN_reports_it(@TempDir directory: File) {
        assertEquals(
            listOf(
                "Replace runBlocking with runTest from kotlinx.coroutines.test: it skips delays and reports " +
                    "coroutines that leak from the test."
            ),
            lintAt(directory, "src/jvmTest/kotlin/LoaderTest.kt")
        )
    }

    @Test
    fun GIVEN_run_blocking_in_main_sources_WHEN_linted_THEN_reports_nothing(@TempDir directory: File) {
        assertEquals(emptyList(), lintAt(directory, "src/main/kotlin/Main.kt"))
    }

    private companion object {
        val CODE = """
            fun load() = runBlocking { runTest { } }
        """.trimIndent()
    }
}
