@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.time

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class DurationOverloadCallTest {
    private val rule = DurationOverloadCall(Config.empty)

    @Test
    fun GIVEN_whole_units_passed_to_duration_calls_WHEN_linted_THEN_reports_each() {
        val code = """
            suspend fun wait(interval: Duration, flow: Flow<Int>) {
                delay(interval.inWholeMilliseconds)
                withTimeout(interval.inWholeSeconds.toLong()) { }
                flow.debounce(interval.inWholeNanoseconds.toInt())
            }
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(3, messages.size)
        assertEquals(
            "Pass the Duration to delay instead of 'interval.inWholeMilliseconds': delay has a Duration overload.",
            messages.first()
        )
    }

    @Test
    fun GIVEN_durations_and_millisecond_boundaries_WHEN_linted_THEN_reports_nothing() {
        val code = """
            suspend fun wait(interval: Duration) {
                delay(interval)
                delay(400)
                delay(interval.inWholeMilliseconds.coerceAtLeast(1))
                delay(interval.toString().length.toLong())
                tween(durationMillis = interval.inWholeMilliseconds.toInt())
                delay(ms())
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
