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
            .main(
                "com.example.jail.jail.usecase",
                "JailPlayerUseCase.kt",
                "class JailPlayerUseCase { operator fun invoke() = Unit }"
            )
            .main("com.example.jail", "Jail.kt", "data class Jail(val id: Int)")
            .test("com.example.jail.jail.model", "JailPlayerUseCaseTest.kt", "class JailPlayerUseCaseTest")
        val reportFile = File(directory, "build/architecture/konsist.tsv")

        main(
            arrayOf(
                "--project-dir", directory.absolutePath,
                "--unit-kind", "context",
                "--unit-path", ":jail",
                "--main", tree.mainDirectory.absolutePath,
                "--main", File(directory, "src/jvmMain/kotlin").absolutePath,
                "--test", tree.testDirectory.absolutePath,
                "--report", reportFile.absolutePath,
            )
        )

        assertEquals(
            listOf(
                tsvLine(
                    "KindShape",
                    "src/main/kotlin/com/example/jail/Jail.kt",
                    "Jail",
                    "Move Jail into a kind package: the unit root holds only packages."
                ),
                tsvLine(
                    "TestMirror",
                    "src/test/kotlin/com/example/jail/jail/model/JailPlayerUseCaseTest.kt",
                    "JailPlayerUseCaseTest",
                    "Move JailPlayerUseCaseTest to com.example.jail.jail.usecase, the package of JailPlayerUseCase."
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
