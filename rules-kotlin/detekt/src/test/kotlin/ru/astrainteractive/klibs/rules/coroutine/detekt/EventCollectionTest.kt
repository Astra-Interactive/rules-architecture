@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class EventCollectionTest {
    private val rule = EventCollection(Config.empty)

    @Test
    fun GIVEN_events_collected_off_the_main_immediate_dispatcher_WHEN_linted_THEN_reports_each() {
        val code = """
            @Composable
            fun CheckoutScreenComposable(viewModel: CheckoutViewModel) {
                LaunchedEffect(viewModel) {
                    viewModel.events.collect { event -> show(event) }
                }
            }

            class CheckoutFragment : Fragment() {
                fun bind(viewModel: CheckoutViewModel) {
                    lifecycleScope.launch { viewModel.checkoutEvents.collectLatest { event -> show(event) } }
                }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Collect these events inside withContext(Dispatchers.Main.immediate) in the LaunchedEffect: the UI " +
                    "handles an event at once, on the frame it arrives.",
                "Collect these events inside repeatOnLifecycle: the UI collects one-off events only while it is " +
                    "started, on Dispatchers.Main.immediate.",
            ),
            rule.lint(code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_events_collected_in_their_lifecycle_and_other_flows_WHEN_linted_THEN_reports_nothing() {
        val code = """
            @Composable
            fun CheckoutScreenComposable(viewModel: CheckoutViewModel) {
                LaunchedEffect(viewModel) {
                    withContext(Dispatchers.Main.immediate) {
                        viewModel.events.collect { event -> show(event) }
                    }
                }
                LaunchedEffect(viewModel) {
                    viewModel.state.collect { state -> render(state) }
                }
            }

            class CheckoutFragment : Fragment() {
                fun bind(viewModel: CheckoutViewModel) {
                    lifecycleScope.launch {
                        repeatOnLifecycle(Lifecycle.State.STARTED) {
                            viewModel.events.collect { event -> show(event) }
                        }
                    }
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
