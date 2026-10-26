@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertFalse
import kotlin.test.assertTrue

internal class SuppressionPolicyTest {
    private fun writeMain(directory: File, path: String, content: String) {
        val file = File(directory, "src/main/kotlin/$path")
        file.parentFile.mkdirs()
        file.writeText(content.trimIndent())
    }

    private fun loadContainerSources(directory: File): UnitSources {
        return SourceTree(directory)
            .main(
                "com.example.model/Container.kt",
                """
                @kotlin.Suppress("OuterRule", "SharedRule")
                class Container {
                    @Suppress("InnerRule")
                    class Inner
                    class Sibling
                }
                """
            )
            .load(UnitKind.CONTEXT)
    }

    @Test
    fun GIVEN_declaration_suppressions_WHEN_evaluated_THEN_matches_only_named_rules(@TempDir directory: File) {
        val sources = loadContainerSources(directory)
        val file = sources.mainFiles.single()
        val outer = file.classes(includeNested = false).single()
        val inner = outer.classes(includeNested = false).first { declaration -> declaration.name == "Inner" }
        val policy = SuppressionPolicy()

        listOf("OuterRule", "SharedRule").forEach { rule ->
            val violation = sources.createViolation(
                rule,
                file,
                outer,
                "Container",
                "message"
            )
            assertTrue(policy.isSuppressed(violation))
        }
        val innerViolation = sources.createViolation(
            "InnerRule",
            file,
            inner,
            "Inner",
            "message"
        )
        assertTrue(policy.isSuppressed(innerViolation))
        val otherViolation = sources.createViolation(
            "OtherRule",
            file,
            inner,
            "Inner",
            "message"
        )
        assertFalse(policy.isSuppressed(otherViolation))
    }

    @Test
    fun GIVEN_enclosing_suppression_WHEN_evaluated_THEN_applies_to_nested_declarations(@TempDir directory: File) {
        val sources = loadContainerSources(directory)
        val file = sources.mainFiles.single()
        val outer = file.classes(includeNested = false).single()
        val declarations = outer.classes(includeNested = false)
        val policy = SuppressionPolicy()

        declarations.forEach { declaration ->
            val violation = sources.createViolation(
                "OuterRule",
                file,
                declaration,
                declaration.name,
                "message"
            )
            assertTrue(policy.isSuppressed(violation))
        }
        val sibling = declarations.first { declaration -> declaration.name == "Sibling" }
        val violation = sources.createViolation(
            "InnerRule",
            file,
            sibling,
            sibling.name,
            "message"
        )
        assertFalse(policy.isSuppressed(violation))
    }

    @Test
    fun GIVEN_file_suppressions_and_external_origin_WHEN_evaluated_THEN_checks_only_matching_source_rules(
        @TempDir directory: File
    ) {
        writeMain(
            directory,
            "com/example/Suppressed.kt",
            """
            @file:Suppress("FileRule", "SharedRule")
            package com.example
            class Suppressed
            """
        )
        val sources = SourceTree(directory)
            .main("com.example/Visible.kt", "class Visible")
            .load(UnitKind.CONTEXT)
        val suppressedFile = sources.mainFiles.first { file -> file.name == "Suppressed" }
        val visibleFile = sources.mainFiles.first { file -> file.name == "Visible" }
        val declaration = suppressedFile.classes(includeNested = false).single()
        val policy = SuppressionPolicy()

        listOf("FileRule", "SharedRule").forEach { rule ->
            val violation = sources.createViolation(
                rule,
                suppressedFile,
                "Suppressed",
                "message"
            )
            assertTrue(policy.isSuppressed(violation))
        }
        val declarationViolation = sources.createViolation(
            "FileRule",
            suppressedFile,
            declaration,
            "Suppressed",
            "message"
        )
        assertTrue(policy.isSuppressed(declarationViolation))
        val otherViolation = sources.createViolation(
            "OtherRule",
            suppressedFile,
            "Suppressed",
            "message"
        )
        assertFalse(policy.isSuppressed(otherViolation))
        val visibleViolation = sources.createViolation(
            "FileRule",
            visibleFile,
            "Visible",
            "message"
        )
        assertFalse(policy.isSuppressed(visibleViolation))
        val externalViolation = sources.createExternalViolation(
            "FileRule",
            "missing/Missing.kt",
            "Missing",
            "message"
        )
        assertFalse(policy.isSuppressed(externalViolation))
    }
}
