@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.compose

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ComposableFileTypeTest {
    private val rule = ComposableFileType(Config.empty)

    @Test
    fun GIVEN_types_beside_composables_WHEN_linted_THEN_reports_each_but_defaults() {
        val code = """
            data class SoftEdge(val edge: Float)

            enum class FadeSide { TOP, BOTTOM }

            private object ScrollFadeValues

            object ButtonDefaults

            interface Fade

            @Composable
            fun ScrollFadeComposable() = Unit
        """.trimIndent()

        val messages = rule.lint(code).map { finding -> finding.message }

        assertEquals(4, messages.size)
        assertEquals(
            "Move 'SoftEdge' out of this composable file: a composable file declares no class; a data class goes " +
                "to model/, shared values to theme/.",
            messages.first()
        )
    }

    @Test
    fun GIVEN_types_without_composables_WHEN_linted_THEN_reports_nothing() {
        val code = """
            data class SoftEdge(val edge: Float)

            class Fade {
                fun render() = Unit
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
