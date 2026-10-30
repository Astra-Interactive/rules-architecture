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
                "End this callbackFlow block with awaitClose { unregister }: the builder throws " +
                    "IllegalStateException at run time when the block completes without it, and the listener leaks.",
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
    fun GIVEN_await_close_that_does_not_end_the_block_WHEN_linted_THEN_reports_each_bridge() {
        val code = """
            fun ExoPlayer.inLaunch(): Flow<Int> = callbackFlow {
                addListener(listener)
                launch { awaitClose { removeListener(listener) } }
            }

            fun ExoPlayer.inBranch(): Flow<Int> = callbackFlow {
                addListener(listener)
                if (isReady) awaitClose { removeListener(listener) }
            }

            fun ExoPlayer.beforeRegistration(): Flow<Int> = callbackFlow {
                awaitClose { removeListener(listener) }
                addListener(listener)
            }
        """.trimIndent()

        assertEquals(3, rule.lint(code).size)
    }

    @Test
    fun GIVEN_bridge_shared_eagerly_WHEN_linted_THEN_reports_the_start() {
        val code = """
            fun ExoPlayer.asShared(scope: CoroutineScope): Flow<Int> = callbackFlow { awaitClose { } }
                .conflate()
                .shareIn(scope, SharingStarted.Eagerly)

            fun ExoPlayer.asState(scope: CoroutineScope): StateFlow<Int> = callbackFlow { awaitClose { } }
                .stateIn(scope = scope, started = SharingStarted.WhileSubscribed(), initialValue = 0)
        """.trimIndent()

        assertEquals(
            listOf(
                "Share this bridge with SharingStarted.WhileSubscribed(): it registers the callback while someone " +
                    "collects and unregisters after the last collector leaves.",
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

            fun ExoPlayer.asGuarded(): Flow<Int> = callbackFlow {
                addListener(listener)
                try {
                    awaitClose()
                } finally {
                    removeListener(listener)
                }
            }

            fun build() = callbackFlow(block)

            fun read() = listOf(1).map { value -> value }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
