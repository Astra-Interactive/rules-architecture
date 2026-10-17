@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.cli

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.test.assertTrue

internal class CliArgumentsTest {
    private val required = listOf(
        "--project-dir",
        "/unit",
        "--unit-kind",
        "entry-point",
        "--report",
        "/unit/report.tsv"
    )

    @Test
    fun GIVEN_every_option_WHEN_parsed_THEN_collects_repeated_directories() {
        val arguments = CliArguments.parse(
            required + listOf(
                "--root-package",
                "com.example",
                "--unit-path",
                ":instances:bukkit",
                "--main",
                "/unit/src/main/kotlin",
                "--main",
                "/unit/src/jvmMain/kotlin",
                "--test",
                "/unit/src/test/kotlin",
            )
        )

        assertEquals(
            CliArguments(
                projectDirectory = File("/unit"),
                unitKind = UnitKind.ENTRY_POINT,
                rootPackage = "com.example",
                unitPath = ":instances:bukkit",
                mainDirectories = listOf(File("/unit/src/main/kotlin"), File("/unit/src/jvmMain/kotlin")),
                testDirectories = listOf(File("/unit/src/test/kotlin")),
                reportFile = File("/unit/report.tsv")
            ),
            arguments
        )
    }

    @Test
    fun GIVEN_only_required_options_WHEN_parsed_THEN_has_no_root_package_and_no_directories() {
        val arguments = CliArguments.parse(required)

        assertNull(arguments.rootPackage)
        assertNull(arguments.unitPath)
        assertEquals(emptyList(), arguments.mainDirectories + arguments.testDirectories)
    }

    @Test
    fun GIVEN_malformed_options_WHEN_parsed_THEN_fails_with_the_reason() {
        val cases = mapOf(
            "Every option takes exactly one value" to required + "--main",
            "Unknown options [--verbose]" to required + listOf("--verbose", "true"),
            "--report is required" to required.dropLast(2),
            "Unknown unit kind 'service'" to listOf(
                "--unit-kind",
                "service",
                "--project-dir",
                "/unit",
                "--report",
                "/r"
            ),
            "--report is given 2 times" to required + listOf("--report", "/other.tsv"),
        )

        cases.forEach { (expectedReason, arguments) ->
            val error = assertFailsWith<IllegalArgumentException> { CliArguments.parse(arguments) }
            assertTrue(
                error.message.orEmpty().startsWith(expectedReason),
                "expected '$expectedReason' but was '${error.message}'"
            )
        }
    }
}
