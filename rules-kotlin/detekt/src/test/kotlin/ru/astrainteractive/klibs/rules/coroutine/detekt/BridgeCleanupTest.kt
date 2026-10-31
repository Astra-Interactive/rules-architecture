@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class BridgeCleanupTest {
    private val rule = BridgeCleanup(Config.empty)

    @Test
    fun GIVEN_cleanups_touching_shared_state_or_launching_WHEN_linted_THEN_reports_each() {
        val code = """
            class PlayerBridge(private val scope: CoroutineScope) {
                private var listener: Listener? = null

                fun states() = callbackFlow {
                    addListener(current)
                    awaitClose {
                        listener = null
                        scope.launch { removeListenerSuspending(current) }
                    }
                }

                suspend fun consent() = suspendCancellableCoroutine { continuation ->
                    continuation.invokeOnCancellation { listener = null }
                }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Keep the awaitClose cleanup to what this bridge registered: it assigns 'listener' declared outside " +
                    "it, which races with the next registration.",
                "Unregister synchronously in awaitClose: a launch there runs on the cancelled scope and dies " +
                    "silently; a suspending unregister runs on a scope the class owns.",
                "Keep the invokeOnCancellation cleanup to what this bridge registered: it assigns 'listener' " +
                    "declared outside it, which races with the next registration.",
            ),
            rule.lint(code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_cleanups_removing_their_own_registration_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun ExoPlayer.states() = callbackFlow {
                val listener = Listener { state -> trySend(state) }
                addListener(listener)
                awaitClose {
                    val removed = removeListener(listener)
                    log(removed)
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_cleanup_assigning_a_variable_of_its_own_bridge_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun LifecycleManager.events(): Flow<Event> = callbackFlow {
                var isEnabled = true
                registerEventHandler { event -> if (isEnabled) trySend(event) }
                awaitClose { isEnabled = false }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
