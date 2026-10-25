@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.coroutine

import dev.detekt.api.Config
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.createEnvironment
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class StateFlowTimeoutTest {
    private val rule = StateFlowTimeout(Config.empty)
    private val environment = createEnvironment()

    @Test
    fun GIVEN_state_flow_reads_inside_timeouts_WHEN_linted_THEN_reports_only_immediate_first_reads() {
        val code = """
            import kotlinx.coroutines.flow.MutableStateFlow
            import kotlinx.coroutines.flow.SharedFlow
            import kotlinx.coroutines.flow.StateFlow
            import kotlinx.coroutines.withTimeout
            import kotlinx.coroutines.withTimeoutOrNull

            suspend fun read(
                state: StateFlow<Int>,
                mutable: MutableStateFlow<Int>,
                shared: SharedFlow<Int>
            ) {
                withTimeout(10) { state.first() }
                withTimeout(10) { kotlinx.coroutines.coroutineScope { state.first() } }
                withTimeoutOrNull(10) { mutable.first() }
                withTimeout(10) { shared.first() }
                withTimeout(10) { state.first { value -> value > 0 } }
                withTimeout(10) { state.value }
            }
        """.trimIndent()

        assertEquals(3, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_state_flow_first_outside_timeout_WHEN_linted_THEN_reports_nothing() {
        val code = """
            import kotlinx.coroutines.flow.StateFlow

            suspend fun read(state: StateFlow<Int>): Int {
                return state.first()
            }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }
}
