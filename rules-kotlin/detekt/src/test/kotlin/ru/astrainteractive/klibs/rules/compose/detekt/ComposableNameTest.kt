@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.compose.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import dev.detekt.test.utils.compileContentForTest
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
    fun GIVEN_composable_word_inside_the_name_WHEN_linted_THEN_suggests_it_only_at_the_end() {
        val code = """
            @Composable
            fun MainComposableScreen() = Column { }
        """.trimIndent()

        assertEquals(
            listOf(
                "Name 'MainComposableScreen' for what it shows, ending in Composable ('MainScreenComposable'): a " +
                    "composable that emits UI is named <Thing>Composable, a screen <Feature>ScreenComposable."
            ),
            rule.lint(code).map(Finding::message)
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
    fun GIVEN_theme_function_in_its_composable_file_WHEN_linted_THEN_reports_nothing() {
        val file = compileContentForTest(THEME_CODE, filename = "AppThemeComposable.kt")

        assertEquals(emptyList(), rule.lint(file).map(Finding::message))
    }

    @Test
    fun GIVEN_theme_function_beside_the_theme_object_WHEN_linted_THEN_reports_its_file() {
        val file = compileContentForTest(THEME_CODE, filename = "AppTheme.kt")

        assertEquals(
            listOf(
                "Move the theme function 'AppTheme' to AppThemeComposable.kt: the theme object sits alone in its own " +
                    "file, the theme function with its CompositionLocals in <Theme>Composable.kt.",
            ),
            rule.lint(file).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_screen_composables_WHEN_linted_THEN_reports_the_one_in_a_file_of_another_name() {
        val code = """
            @Composable
            fun CheckoutScreenComposable() = Unit

            @Composable
            fun PaywallScreenComposable() = Unit
        """.trimIndent()
        val file = compileContentForTest(code, filename = "CheckoutScreenComposable.kt")

        assertEquals(
            listOf(
                "Move the screen 'PaywallScreenComposable' to PaywallScreenComposable.kt: a screen composable's " +
                    "file carries its name.",
            ),
            rule.lint(file).map(Finding::message)
        )
    }

    private companion object {
        val THEME_CODE = """
            package x.core.ui.theme

            @Composable
            fun AppTheme(content: @Composable () -> Unit) {
                content.invoke()
            }
        """.trimIndent()
    }
}
