@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.time.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class SystemTimeCallTest {
    private val rule = SystemTimeCall(Config.empty)

    @Test
    fun GIVEN_system_time_calls_WHEN_linted_THEN_reports_each() {
        val code = """
            fun measure(): Long = System.currentTimeMillis() - java.lang.System.nanoTime()
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(2, messages.size)
        assertEquals(
            "Replace System.currentTimeMillis() with Clock.System.now() from kotlin.time, injected as a Clock " +
                "where the code must be testable.",
            messages.first()
        )
        assertEquals(
            "Replace System.nanoTime() with TimeSource.Monotonic from kotlin.time to measure an elapsed time.",
            messages.last()
        )
    }

    @Test
    fun GIVEN_clock_and_other_system_calls_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun read() = listOf(Clock.System.now(), System.getenv("HOME"), System.out, timer.nanoTime())
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
