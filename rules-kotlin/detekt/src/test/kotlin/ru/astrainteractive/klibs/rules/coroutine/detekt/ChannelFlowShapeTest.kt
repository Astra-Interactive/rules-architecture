@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.createEnvironment
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ChannelFlowShapeTest {
    private val rule = ChannelFlowShape(Config.empty)

    private val environment = createEnvironment()

    @Test
    fun GIVEN_scope_helper_with_listener_registration_WHEN_linted_THEN_still_reports_missing_cleanup() {
        val code = """
            import kotlinx.coroutines.CoroutineScope
            import kotlinx.coroutines.flow.channelFlow

            fun CoroutineScope.startWorker() = Unit
            fun registerListener(listener: () -> Unit) = Unit

            fun numbers() = channelFlow<Int> {
                startWorker()
                registerListener { }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Bridge 'registerListener' with callbackFlow ending in awaitClose { unregister }: channelFlow " +
                    "completes with its block, the listener stays registered and its next send throws " +
                    "ClosedSendChannelException."
            ),
            rule.lintWithContext(environment, code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_local_helpers_and_non_scope_extensions_WHEN_linted_THEN_still_suggests_flow() {
        val code = """
            import kotlinx.coroutines.flow.channelFlow

            fun numbers() = channelFlow<Int> {
                fun number(): Int = 1
                send(number().inc())
                println("ready")
            }
        """.trimIndent()

        assertEquals(1, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_unresolved_arguments_in_incomplete_code_WHEN_linted_THEN_does_not_crash() {
        val code = """
            import kotlinx.coroutines.flow.channelFlow

            fun numbers() = channelFlow<Int> {
                missing(, unknown)
                missing { }
            }
        """.trimIndent()

        assertEquals(1, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_producer_extension_launching_through_a_helper_WHEN_linted_THEN_keeps_channel_flow() {
        val code = """
            import kotlinx.coroutines.CoroutineScope
            import kotlinx.coroutines.channels.ProducerScope
            import kotlinx.coroutines.flow.channelFlow
            import kotlinx.coroutines.launch

            fun numbers() = channelFlow<Int> { serve() }

            private suspend fun ProducerScope<Int>.serve() {
                observe()
                send(1)
            }

            private fun CoroutineScope.observe() = launch { println("observing") }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_scope_passed_to_an_opaque_helper_WHEN_linted_THEN_does_not_claim_absence_of_coroutines() {
        val code = """
            import kotlinx.coroutines.CoroutineScope
            import kotlinx.coroutines.flow.channelFlow

            interface Worker {
                fun start(scope: CoroutineScope)
            }

            fun numbers(worker: Worker) = channelFlow<Int> {
                val producer = this
                worker.start(producer)
                send(1)
            }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_explicit_receiver_on_a_scope_helper_WHEN_linted_THEN_keeps_channel_flow() {
        val code = """
            import kotlinx.coroutines.CoroutineScope
            import kotlinx.coroutines.flow.channelFlow
            import kotlinx.coroutines.launch

            private fun CoroutineScope.startWorker() = launch { println("working") }

            fun numbers() = channelFlow<Int> {
                this.startWorker()
                send(1)
            }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_scope_helper_and_await_close_WHEN_linted_THEN_does_not_suggest_callback_flow() {
        val code = """
            import kotlinx.coroutines.CoroutineScope
            import kotlinx.coroutines.channels.awaitClose
            import kotlinx.coroutines.flow.channelFlow
            import kotlinx.coroutines.launch

            private fun CoroutineScope.startWorker() = launch { println("working") }

            fun numbers() = channelFlow<Int> {
                startWorker()
                awaitClose { }
            }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_helper_without_scope_access_WHEN_linted_THEN_still_suggests_flow() {
        val code = """
            import kotlinx.coroutines.flow.channelFlow

            private fun number(): Int = 1

            fun numbers() = channelFlow<Int> {
                send(number())
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Write this producer as flow { }: channelFlow is for a producer that launches coroutines of its own."
            ),
            rule.lintWithContext(environment, code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_resolved_await_close_without_scope_helpers_WHEN_linted_THEN_still_suggests_callback_flow() {
        val code = """
            import kotlinx.coroutines.channels.awaitClose
            import kotlinx.coroutines.flow.channelFlow

            fun numbers() = channelFlow<Int> {
                send(1)
                awaitClose { }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Write this producer as callbackFlow { }: it launches no coroutine of its own and holds what it " +
                    "sends until awaitClose, which flow { } has no place for."
            ),
            rule.lintWithContext(environment, code).map(Finding::message)
        )
    }

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
            rule.lintWithContext(environment, code).map(Finding::message)
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
            rule.lintWithContext(environment, code).map(Finding::message)
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

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_channel_flow_holding_a_resource_until_await_close_WHEN_linted_THEN_suggests_callback_flow() {
        val code = """
            fun bots(token: String): Flow<Application> = channelFlow {
                val application = Application()
                application.register(token)
                send(application)
                awaitClose { application.close() }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Write this producer as callbackFlow { }: it launches no coroutine of its own and holds what it " +
                    "sends until awaitClose, which flow { } has no place for.",
            ),
            rule.lintWithContext(environment, code).map(Finding::message)
        )
    }
}
