@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.compose.detekt

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
                    Stage.OFFER -> {
                        val offer = rememberOffer()
                        OfferComposable(offer)
                    }
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

    @Test
    fun GIVEN_a_block_of_one_call_with_three_arguments_WHEN_linted_THEN_reports_nothing() {
        val code = """
            @Composable
            fun TunerStateComposable(state: TunerState, onStop: () -> Unit) {
                when (state) {
                    is TunerState.Listening -> {
                        ListeningLayoutComposable(
                            frequency = state.frequency,
                            detectedNote = state.detectedNote,
                            onStopListening = onStop
                        )
                    }
                    is TunerState.Granted -> {
                        screen.GrantedLayoutComposable(
                            onStart = onStop,
                            modifier = Modifier
                        ) {
                            HintComposable()
                        }
                    }
                    TunerState.NoPermission -> NoPermissionLayoutComposable()
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_a_wrapped_call_outside_braces_or_a_block_of_a_short_call_WHEN_linted_THEN_reports_each() {
        val code = """
            @Composable
            fun TunerStateComposable(state: TunerState, onStop: () -> Unit) {
                when (state) {
                    is TunerState.Listening -> ListeningLayoutComposable(
                        frequency = state.frequency,
                        detectedNote = state.detectedNote,
                        onStopListening = onStop
                    )
                    is TunerState.Granted -> {
                        GrantedLayoutComposable(
                            onStart = onStop
                        )
                    }
                }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Put this call in braces: a branch whose call takes three or more arguments is a block `-> { }` " +
                    "holding that call alone.",
                "Make this branch one call of its own @Composable, so the when reads as a dispatch table.",
            ),
            rule.lint(code).map(Finding::message)
        )
    }
}
