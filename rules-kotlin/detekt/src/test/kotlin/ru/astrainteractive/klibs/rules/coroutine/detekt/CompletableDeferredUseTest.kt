@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class CompletableDeferredUseTest {
    private val rule = CompletableDeferredUse(Config.empty)

    @Test
    fun GIVEN_deferred_awaited_where_created_and_deferred_flags_WHEN_linted_THEN_reports_each() {
        val code = """
            class ConsentGateway(private val sdk: Sdk) {
                private val ready = CompletableDeferred<Unit>()
                private val isLoaded: CompletableDeferred<Boolean> = CompletableDeferred()

                suspend fun requestConsent(): Boolean {
                    val answer = CompletableDeferred<Boolean>()
                    sdk.request { granted -> answer.complete(granted) }
                    return answer.await()
                }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Hold the readiness in a StateFlow instead of the CompletableDeferred 'ready': a deferred is " +
                    "completed once and is no state.",
                "Hold the readiness in a StateFlow instead of the CompletableDeferred 'isLoaded': a deferred is " +
                    "completed once and is no state.",
                "Bridge this result with suspendCancellableCoroutine instead of the CompletableDeferred 'answer': " +
                    "a deferred created and awaited in one function replaces a suspend function.",
            ),
            rule.lint(code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_deferred_answered_by_another_component_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class ConfirmDialog {
                private val answer = CompletableDeferred<Choice>()

                fun onChoose(choice: Choice) {
                    answer.complete(choice)
                }

                suspend fun awaitChoice(): Choice = answer.await()
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_deferred_gate_awaited_by_another_coroutine_WHEN_linted_THEN_reports_only_the_one_awaited_in_place() {
        val code = """
            class ChannelTest {
                @Test
                fun GIVEN_stuck_receiver_WHEN_resumed_THEN_gets_newest(): TestResult {
                    return runTest {
                        val gate = CompletableDeferred<Unit>()
                        backgroundScope.launch { channel.collect { event -> gate.await() } }
                        gate.complete(Unit)
                    }
                }

                @Test
                fun GIVEN_answer_WHEN_awaited_THEN_reads_it(): TestResult {
                    return runTest {
                        val answer = CompletableDeferred<Int>()
                        sdk.request { value -> answer.complete(value) }
                        answer.await()
                    }
                }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Bridge this result with suspendCancellableCoroutine instead of the CompletableDeferred 'answer': " +
                    "a deferred created and awaited in one function replaces a suspend function.",
            ),
            rule.lint(code).map(Finding::message)
        )
    }
}
