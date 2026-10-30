@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ChannelFlowShapeTest {
    private val rule = ChannelFlowShape(Config.empty)

    @Test
    fun GIVEN_channel_flow_registering_a_listener_WHEN_linted_THEN_reports_the_bridge() {
        val code = """
            fun ExoPlayer.asStates(): Flow<Int> = channelFlow {
                launch { send(1) }
                addListener(listener)
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Bridge 'addListener' with callbackFlow ending in awaitClose { unregister }: channelFlow completes " +
                    "with its block, the listener stays registered and its next send throws " +
                    "ClosedSendChannelException.",
            ),
            rule.lint(code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_channel_flow_without_coroutines_WHEN_linted_THEN_suggests_flow() {
        val code = """
            fun numbers(): Flow<Int> = channelFlow {
                send(1)
                send(2)
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Write this producer as flow { }: channelFlow is for a producer that launches coroutines of its own.",
            ),
            rule.lint(code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_channel_flows_in_shape_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun merged(first: Flow<Int>, second: Flow<Int>): Flow<Int> = channelFlow {
                launch { first.collect { value -> send(value) } }
                launch { second.collect { value -> send(value) } }
            }

            fun ExoPlayer.closed(): Flow<Int> = channelFlow {
                launch { send(1) }
                registerCallback(callback)
                awaitClose { unregisterCallback(callback) }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
