@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.compose

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ComposableWhenBranchTest {
    private val rule = ComposableWhenBranch(Config.empty)

    @Test
    fun GIVEN_multi_line_branches_in_a_composable_WHEN_linted_THEN_reports_each() {
        val code = """
            @Composable
            fun SplashComposable(stage: Stage) {
                when (stage) {
                    Stage.SUCCESS -> Column {
                        Spacer()
                    }
                    Stage.OFFER -> {
                        OfferComposable()
                    }
                    Stage.REVEAL -> RevealComposable()
                }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Make this branch one call of its own @Composable, so the when reads as a dispatch table.",
                "Make this branch one call of its own @Composable, so the when reads as a dispatch table.",
            ),
            rule.lint(code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_one_line_branches_or_plain_functions_WHEN_linted_THEN_reports_nothing() {
        val code = """
            @Composable
            fun SplashComposable(stage: Stage) {
                when (stage) {
                    Stage.SUCCESS -> SuccessComposable()
                    else -> {}
                }
            }

            fun select(stage: Stage): Int {
                return when (stage) {
                    Stage.SUCCESS -> {
                        1
                    }
                    else -> 2
                }
            }

            val top = when (1) {
                1 -> {
                    2
                }
                else -> 3
            }

            fun empty(stage: Stage) {
                when (stage) {
                    else ->
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
