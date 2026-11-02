@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.createEnvironment
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class BridgeCleanupTest {
    private val rule = BridgeCleanup(Config.empty)
    private val environment = createEnvironment()

    @Test
    fun GIVEN_cleanups_touching_shared_state_or_launching_WHEN_linted_THEN_reports_each() {
        val code = IMPORTS + """
            class PlayerBridge(private val scope: CoroutineScope) {
                private var listener: String? = null

                fun states() = callbackFlow<Unit> {
                    awaitClose {
                        listener = null
                        launch { delay(1) }
                        scope.launch { delay(1) }
                    }
                }

                suspend fun consent(): Unit = suspendCancellableCoroutine { continuation ->
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
            rule.lintWithContext(environment, code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_cleanups_removing_their_own_registration_WHEN_linted_THEN_reports_nothing() {
        val code = IMPORTS + """
            class Registration(var active: Boolean)

            fun states() = callbackFlow<Unit> {
                val registration = Registration(true)
                awaitClose {
                    registration.active = false
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_cleanup_assigning_a_variable_of_its_own_bridge_WHEN_linted_THEN_reports_nothing() {
        val code = IMPORTS + """
            class Bridge {
                private var isEnabled = true

                fun events() = callbackFlow<Unit> {
                    var isEnabled = this@Bridge.isEnabled
                    awaitClose { isEnabled = false }
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_owned_atomic_cleanup_and_unknown_injected_scope_WHEN_checked_THEN_does_not_assume_bridge_cancellation() {
        val code = IMPORTS + """
            @OptIn(DelicateCoroutinesApi::class)
            class Bridge(private val unknown: CoroutineScope) {
                private val cleanupScope = CoroutineScope(SupervisorJob())

                fun events() = callbackFlow<Unit> {
                    awaitClose {
                        cleanupScope.launch(start = CoroutineStart.ATOMIC) {
                            withContext(NonCancellable) { delay(1) }
                        }
                        unknown.launch { delay(1) }
                        with(unknown) { launch { delay(1) } }
                    }
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_explicit_bridge_scope_and_alias_WHEN_launched_in_cleanup_THEN_reports_both() {
        val code = IMPORTS + """
            fun events() = callbackFlow<Unit> {
                val bridgeScope = this
                val untypedScope: Any = this
                awaitClose {
                    this@callbackFlow.launch { delay(1) }
                    bridgeScope.launch { delay(1) }
                    if (untypedScope is CoroutineScope) untypedScope.launch { delay(1) }
                }
            }
        """.trimIndent()

        assertEquals(3, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_alias_of_external_state_WHEN_written_in_cleanup_THEN_still_reports_shared_state() {
        val code = IMPORTS + """
            class State(var active: Boolean)

            class Bridge(private val shared: State) {
                private var listener: String? = null
                private val values = intArrayOf(1)

                fun events() = callbackFlow<Unit> {
                    val alias = shared
                    val local = State(true)
                    awaitClose {
                        alias.active = false
                        this@Bridge.listener = null
                        values[0] = 0
                        local.active = false
                    }
                }
            }
        """.trimIndent()
        val assignments = rule.lintWithContext(environment, code).map { finding ->
            finding.message.substringAfter("assigns '").substringBefore("'")
        }

        assertEquals(
            listOf(
                "alias.active",
                "this@Bridge.listener",
                "values[0]"
            ),
            assignments
        )
    }

    @Test
    fun GIVEN_unrelated_cleanup_name_WHEN_checked_THEN_does_not_apply_coroutine_rules() {
        val code = """
            var state = 1
            fun awaitClose(block: () -> Unit) = block()
            fun close() { awaitClose { state = 0 } }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    private companion object {
        val IMPORTS = """
            import kotlinx.coroutines.*
            import kotlinx.coroutines.channels.awaitClose
            import kotlinx.coroutines.flow.callbackFlow

        """.trimIndent() + "\n"
    }
}
