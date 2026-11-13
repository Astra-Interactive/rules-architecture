@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.cli.model

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Vocabulary
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
                "--entry-point-word",
                "slot",
                "--entry-point-word",
                "router",
                "--technology-word",
                "mongo",
                "--plural-concept",
                "goods",
                "--platform-class",
                "WatchFaceService",
                "--extension-kind",
                "slot",
                "--dependency",
                "/data/dao/src/main/kotlin",
                "--android-namespace",
                "com.example",
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
                mainSourceSets = emptyMap(),
                testSourceSets = emptyMap(),
                reportFile = File("/unit/report.tsv"),
                vocabulary = Vocabulary(
                    entryPointWords = setOf("slot", "router"),
                    technologyWords = setOf("mongo"),
                    pluralConcepts = setOf("goods"),
                    platformClasses = setOf("WatchFaceService"),
                    extensionKinds = setOf("slot"),
                    roleKinds = emptySet()
                ),
                dependencyDirectories = listOf(File("/data/dao/src/main/kotlin")),
                androidNamespace = "com.example",
                unitContext = null
            ),
            arguments
        )
    }

    @Test
    fun GIVEN_source_set_qualified_directories_WHEN_parsed_THEN_keeps_names_and_files() {
        val arguments = CliArguments.parse(
            required + listOf(
                "--main",
                "commonMain=/unit/src/commonMain/kotlin",
                "--test",
                "commonTest=/unit/src/commonTest/kotlin"
            )
        )

        assertEquals(
            mapOf("commonMain" to File("/unit/src/commonMain/kotlin")),
            arguments.mainSourceSets
        )
        assertEquals(
            mapOf("commonTest" to File("/unit/src/commonTest/kotlin")),
            arguments.testSourceSets
        )
        assertEquals(
            listOf(File("/unit/src/commonMain/kotlin")),
            arguments.mainDirectories
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

    @Test
    fun GIVEN_repeated_role_kind_options_WHEN_parsed_THEN_collects_them_into_the_vocabulary() {
        val arguments = CliArguments.parse(
            required + listOf(
                "--role-kind",
                "limiter",
                "--role-kind",
                "codec"
            )
        )

        assertEquals(setOf("limiter", "codec"), arguments.vocabulary.roleKinds)
    }

    @Test
    fun GIVEN_role_kind_that_is_a_fixed_technology_or_entry_point_word_WHEN_parsed_THEN_fails_with_the_reason() {
        val projectWords = listOf(
            "--technology-word",
            "mongo",
            "--entry-point-word",
            "widget",
            "--extension-kind",
            "panel"
        )
        val conflictingWords = listOf(
            "api",
            "model",
            "usecase",
            "util",
            "di",
            "theme",
            "stub",
            "fake",
            "table",
            "entity",
            "database",
            "warmup",
            "migration",
            "error",
            "composable",
            "exposed",
            "memory",
            "mongo",
            "command",
            "slot",
            "widget",
            "panel"
        )

        conflictingWords.forEach { word ->
            val error = assertFailsWith<IllegalArgumentException>(word) {
                CliArguments.parse(required + projectWords + listOf("--role-kind", word))
            }
            assertEquals(
                "--role-kind '$word' is a fixed kind, a technology or an entry-point word of the project; a role " +
                    "kind is named by a word of its own.",
                error.message
            )
        }
    }

    @Test
    fun GIVEN_role_kind_that_the_document_renames_WHEN_parsed_THEN_fails_with_the_reason() {
        listOf(
            "manager",
            "helper",
            "controller",
            "utils"
        ).forEach { word ->
            val error = assertFailsWith<IllegalArgumentException>(word) {
                CliArguments.parse(required + listOf("--role-kind", word))
            }
            assertEquals(
                "--role-kind '$word' names no role: a class named ...${word.replaceFirstChar(Char::uppercase)} is " +
                    "renamed, not given a kind.",
                error.message
            )
        }
    }

    @Test
    fun GIVEN_role_kind_that_is_a_forbidden_package_name_WHEN_parsed_THEN_fails_with_the_reason() {
        listOf(
            "factory",
            "adapter",
            "repository",
            "validation",
            "dto"
        ).forEach { word ->
            val error = assertFailsWith<IllegalArgumentException>(word) {
                CliArguments.parse(required + listOf("--role-kind", word))
            }
            assertEquals(
                "--role-kind '$word' is a forbidden package name: a package is named for a concept or a kind, " +
                    "never a layer or a filler word.",
                error.message
            )
        }
    }

    @Test
    fun GIVEN_role_kind_whose_classes_another_check_places_WHEN_parsed_THEN_fails_with_the_reason() {
        val error = assertFailsWith<IllegalArgumentException> {
            CliArguments.parse(required + listOf("--role-kind", "module"))
        }

        assertEquals(
            "--role-kind 'module' names classes another check places: a class named ...Module belongs to di.",
            error.message
        )
    }

    @Test
    fun GIVEN_role_kind_with_capitals_or_padding_WHEN_parsed_THEN_fails_with_the_reason() {
        listOf(
            "Limiter",
            "Model",
            " codec",
            "codec "
        ).forEach { word ->
            val error = assertFailsWith<IllegalArgumentException>(word) {
                CliArguments.parse(required + listOf("--role-kind", word))
            }
            assertEquals(
                "--role-kind '$word' must be lower-case and trimmed: a role kind is a package segment, the last " +
                    "word of its classes in lower case.",
                error.message
            )
        }
    }
}
