@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.cli

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import java.io.File
import kotlin.test.assertEquals

internal class KonsistCliTest {
    private fun tsvLine(vararg fields: String): String = fields.joinToString(separator = TAB.toString())

    @Test
    fun GIVEN_unit_with_violations_WHEN_run_THEN_writes_them_to_the_report(@TempDir directory: File) {
        val tree = SourceTree(directory)
            .main("com.example.jail", "JailPlayer.kt", "class JailPlayer { operator fun invoke() = Unit }")
            .main("com.example.jail", "Jail.kt", "data class Jail(val id: Int)")
            .test("com.example.jail.model", "JailPlayerTest.kt", "class JailPlayerTest")
        val reportFile = File(directory, "build/architecture/konsist.tsv")

        main(
            arrayOf(
                "--project-dir", directory.absolutePath,
                "--unit-kind", "context",
                "--main", tree.mainDirectory.absolutePath,
                "--main", File(directory, "src/jvmMain/kotlin").absolutePath,
                "--test", tree.testDirectory.absolutePath,
                "--report", reportFile.absolutePath,
            )
        )

        assertEquals(
            listOf(
                tsvLine(
                    "LooseRootFile",
                    "src/main/kotlin/com/example/jail/Jail.kt",
                    "Jail",
                    "Move Jail to a role package: the context root holds only use cases, the <Context>Api and its " +
                        "Default<Context>Api."
                ),
                tsvLine(
                    "TestMirror",
                    "src/test/kotlin/com/example/jail/model/JailPlayerTest.kt",
                    "JailPlayerTest",
                    "Move JailPlayerTest to com.example.jail, the package of JailPlayer."
                ),
            ),
            reportFile.readLines()
        )
    }

    @Test
    fun GIVEN_unit_without_sources_WHEN_run_THEN_writes_an_empty_report(@TempDir directory: File) {
        val reportFile = File(directory, "report.tsv")

        main(
            arrayOf(
                "--project-dir",
                directory.absolutePath,
                "--unit-kind",
                "library",
                "--root-package",
                "com.example",
                "--report",
                reportFile.absolutePath,
            )
        )

        assertEquals("", reportFile.readText())
    }

    private companion object {
        const val TAB = '\t'
    }
}
