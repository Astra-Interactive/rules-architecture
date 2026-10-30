@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.test.lint
import dev.detekt.test.utils.compileForTest
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.io.File
import kotlin.test.assertEquals

internal class ThreadPrimitiveTest {
    private val rule = ThreadPrimitive(Config.empty)

    private fun lintAt(file: File): Int {
        file.parentFile.mkdirs()
        file.writeText(CODE)
        return rule.lint(compileForTest(file.toPath())).size
    }

    @Test
    fun GIVEN_threads_in_main_sources_WHEN_linted_THEN_reports_each(@TempDir directory: File) {
        assertEquals(3, lintAt(File(directory, "src/main/kotlin/x/Worker.kt")))
    }

    @Test
    fun GIVEN_threads_in_test_sources_WHEN_linted_THEN_reports_nothing(@TempDir directory: File) {
        assertEquals(0, lintAt(File(directory, "src/test/kotlin/x/WorkerTest.kt")))
    }

    private companion object {
        val CODE = """
            package x

            fun work(job: Job) {
                Thread { job.run() }.start()
                Thread.sleep(10)
                java.lang.Thread.yield()
                job.run()
            }
        """.trimIndent()
    }
}
