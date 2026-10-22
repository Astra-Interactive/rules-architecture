@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.coroutine

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class MutableStateFlowEmitTest {
    private val rule = MutableStateFlowEmit(Config.empty)

    @Test
    fun GIVEN_emit_on_mutable_state_flows_WHEN_linted_THEN_reports_each() {
        val code = """
            private val global = MutableStateFlow(0)

            class SplashViewModel {
                private val state: MutableStateFlow<Int> = createState()
                private val other = MutableStateFlow<String>("")

                suspend fun finish() {
                    state.emit(1)
                    this.other.tryEmit("")
                    global.emit(2)
                }
            }
        """.trimIndent()

        val messages = rule.lint(code).map { finding -> finding.message }

        assertEquals(3, messages.size)
        assertEquals(
            "'state' is a MutableStateFlow: write it with '.value =', or with update { } when the new value " +
                "depends on the current one, never with emit.",
            messages.first()
        )
    }

    @Test
    fun GIVEN_emit_on_other_flows_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class SplashViewModel(private val holder: Holder) {
                private val events = MutableSharedFlow<Int>()
                private val counter = 0
                private val created = createFlow()

                suspend fun finish(flow: MutableStateFlow<Int>) {
                    events.emit(1)
                    flow.emit(1)
                    created.emit(1)
                    holder.state.emit(1)
                    emit(1)
                    counter.toString()
                    listOf(1).first().emit(2)
                    emit(1).let { value -> value }
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
