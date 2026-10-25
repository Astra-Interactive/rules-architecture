@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.multiplatform.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import dev.detekt.test.utils.compileContentForTest
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ActualFileSuffixTest {
    private val rule = ActualFileSuffix(Config.empty)

    @Test
    fun GIVEN_actual_declaration_in_a_plain_file_WHEN_linted_THEN_reports_it() {
        val findings = rule.lint(compileContentForTest(ACTUAL_CODE, filename = "Clipboard.kt"))

        assertEquals(
            listOf(
                "Name the file of an actual declaration with its platform: Clipboard.<platform>.kt, such as " +
                    "Clipboard.android.kt or Clipboard.jvm.kt."
            ),
            findings.map(Finding::message)
        )
    }

    @Test
    fun GIVEN_suffixed_file_or_no_actual_WHEN_linted_THEN_reports_nothing() {
        val suffixed = rule.lint(compileContentForTest(ACTUAL_CODE, filename = "Clipboard.android.kt"))
        val common = rule.lint(compileContentForTest("expect class Clipboard", filename = "Clipboard.kt"))

        assertEquals(0, suffixed.size + common.size)
    }

    private companion object {
        val ACTUAL_CODE = """
            actual class Clipboard {
                actual fun copy(text: String) = Unit
            }
        """.trimIndent()
    }
}
