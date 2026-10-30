@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class SharedFlowEventTest {
    private val rule = SharedFlowEvent(Config.empty)

    @Test
    fun GIVEN_view_model_shared_flows_and_unbuffered_try_emit_WHEN_linted_THEN_reports_each() {
        val code = """
            class CheckoutViewModel {
                private val eventFlow = MutableSharedFlow<CheckoutEvent>()
                val events: SharedFlow<CheckoutEvent> = eventFlow
            }

            class EventBus {
                private val events = MutableSharedFlow<Event>(replay = 0)

                fun publish(event: Event) {
                    this.events.tryEmit(event)
                }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Send the one-off events of 'CheckoutViewModel' through Channel(Channel.BUFFERED).receiveAsFlow(): " +
                    "a SharedFlow drops an event while the UI does not collect.",
                "Send the one-off events of 'CheckoutViewModel' through Channel(Channel.BUFFERED).receiveAsFlow(): " +
                    "a SharedFlow drops an event while the UI does not collect.",
                "Give 'events' an extraBufferCapacity: tryEmit on a MutableSharedFlow without a buffer drops every " +
                    "value.",
            ),
            rule.lint(code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_buffered_or_suspending_shared_flows_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class EventBus {
                private val buffered = MutableSharedFlow<Event>(extraBufferCapacity = 64)
                private val positional = MutableSharedFlow<Event>(0, 64)
                private val suspending = MutableSharedFlow<Event>()
                val shared: SharedFlow<Event> = buffered
                private val state = MutableStateFlow(0)
                private val other = createFlow()

                fun publish(event: Event) {
                    buffered.tryEmit(event)
                    positional.tryEmit(event)
                    tryEmit(event)
                    other.tryEmit(event)
                    state.value = 1
                }

                suspend fun send(event: Event) {
                    suspending.emit(event)
                }
            }

            fun local() {
                val events = MutableSharedFlow<Int>()
                events.tryEmit(1)
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
