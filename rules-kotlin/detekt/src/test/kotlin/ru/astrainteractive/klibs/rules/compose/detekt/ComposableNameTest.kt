@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.compose.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ComposableNameTest {
    private val rule = ComposableName(Config.empty)

    @Test
    fun GIVEN_ui_composables_without_the_suffix_WHEN_linted_THEN_reports_each() {
        val code = """
            @Composable
            internal fun OrderSummary(title: String) {
                Text(title)
            }

            @Composable
            fun CheckoutScreen(): Unit = Column { }

            @Composable
            private fun GlassButton() = Box { }
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(3, messages.size)
        assertEquals(
            "Name 'OrderSummary' for what it shows, ending in Composable ('OrderSummaryComposable'): a composable " +
                "that emits UI is named <Thing>Composable, a screen <Feature>ScreenComposable.",
            messages.first()
        )
    }

    @Test
    fun GIVEN_suffixed_value_returning_and_exempt_composables_WHEN_linted_THEN_reports_nothing() {
        val code = """
            @Composable
            internal fun OrderSummaryComposable() = Unit

            @Composable
            private fun resolveTitleStyle(size: Int): TextStyle = TextStyle()

            @Composable
            private fun rememberShortcuts(): List<Int> = remember { listOf(1) }

            @Composable
            private fun OrderSummaryPreviewSheet() = Unit

            @Preview
            @Composable
            private fun OrderSummaryCenterPreview() = Unit

            @Composable
            fun Modifier.glass(): Modifier = this

            fun OrderSummary() = Unit

            class CheckoutComponent : Component {
                @Composable
                override fun Render() = Unit
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lint(code).map(Finding::message))
    }

    @Test
    fun GIVEN_theme_function_WHEN_linted_THEN_reports_nothing() {
        val code = """
            package x.core.ui.theme

            @Composable
            fun AppTheme(content: @Composable () -> Unit) {
                content.invoke()
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lint(code).map(Finding::message))
    }
}
