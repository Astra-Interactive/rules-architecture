@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class CallbackFlowShapeTest {
    private val rule = CallbackFlowShape(Config.empty)

    @Test
    fun GIVEN_bridges_out_of_shape_WHEN_linted_THEN_reports_each() {
        val code = """
            fun ExoPlayer.asStates(): Flow<Int> {
                return callbackFlow {
                    addListener(listener)
                    launch { send(1) }
                }.buffer(Channel.RENDEZVOUS)
            }

            fun ExoPlayer.asEvents(): Flow<Int> = callbackFlow { awaitClose { } }.buffer(capacity = 0)
        """.trimIndent()

        assertEquals(
            listOf(
                "Unregister the callback in awaitClose { } at the end of this callbackFlow: the builder fails " +
                    "without it and the listener leaks.",
                "Call trySend(value) from the callback instead of launch { send(value) }: a coroutine per event " +
                    "loses their order.",
                "Keep the default buffer, Channel.UNLIMITED or conflate() instead of a rendezvous buffer: the " +
                    "producer can deadlock and trySend fails while the collector is busy.",
                "Keep the default buffer, Channel.UNLIMITED or conflate() instead of a rendezvous buffer: the " +
                    "producer can deadlock and trySend fails while the collector is busy.",
            ),
            rule.lint(code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_bridges_in_shape_and_other_calls_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun ExoPlayer.asStates(): Flow<Int> {
                return callbackFlow {
                    val listener = Listener { state -> trySend(state) }
                    launch { log() }
                    launch(context)
                    addListener(listener)
                    awaitClose { removeListener(listener) }
                }.buffer(Channel.UNLIMITED)
            }

            fun build() = callbackFlow(block)

            fun read() = listOf(1).map { value -> value }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
