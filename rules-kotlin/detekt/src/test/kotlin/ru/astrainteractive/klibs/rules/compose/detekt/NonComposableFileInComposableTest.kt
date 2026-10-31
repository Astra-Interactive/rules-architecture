@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.compose.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import dev.detekt.test.utils.compileForTest
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.io.File
import kotlin.test.assertEquals

internal class NonComposableFileInComposableTest {
    private val rule = NonComposableFileInComposable(Config.empty)

    @Test
    fun GIVEN_plain_functions_in_a_composable_package_WHEN_linted_THEN_reports_the_file() {
        val code = """
            package com.example.paywall.impl.composable.layout

            fun DrawScope.drawCard() = Unit
        """.trimIndent()

        assertEquals(
            listOf(
                "Move Test.kt out of the composable package: a file with no @Composable function is a util file, " +
                    "util/<Subject>Ext.kt for extensions or a file named after its function."
            ),
            rule.lint(code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_composable_files_and_other_packages_WHEN_linted_THEN_reports_nothing() {
        val composable = """
            package com.example.paywall.impl.composable

            @Composable
            fun CardComposable() = Unit
        """.trimIndent()
        val util = """
            package com.example.paywall.impl.util

            fun DrawScope.drawCard() = Unit
        """.trimIndent()
        val empty = "package com.example.paywall.impl.composable"

        assertEquals(0, rule.lint(composable).size + rule.lint(util).size + rule.lint(empty).size)
    }

    @Test
    fun GIVEN_test_in_a_composable_package_WHEN_linted_THEN_reports_nothing(@TempDir directory: File) {
        val code = """
            package com.example.paywall.impl.composable

            class CardComposableTest {
                fun GIVEN_card_WHEN_shown_THEN_has_title() = Unit
            }
        """.trimIndent()
        val findings = listOf(
            "src/test/kotlin/com/example/paywall/impl/composable/CardComposableTest.kt",
            "src/commonTest/kotlin/com/example/paywall/impl/composable/CardComposableTest.kt",
        ).sumOf { path ->
            val file = File(directory, path)
            file.parentFile.mkdirs()
            file.writeText(code)
            rule.lint(compileForTest(file.toPath())).size
        }

        assertEquals(0, findings)
    }
}
