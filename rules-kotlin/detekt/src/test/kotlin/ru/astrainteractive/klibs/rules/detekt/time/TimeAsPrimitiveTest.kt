@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.time

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class TimeAsPrimitiveTest {
    private val rule = TimeAsPrimitive(Config.empty)

    @Test
    fun GIVEN_time_named_primitives_WHEN_linted_THEN_reports_each() {
        val code = """
            class Session(
                val createdAt: Long,
                val expiresAt: String?,
                val timeoutSeconds: Int
            ) {
                val delay: Double = 1.0

                fun schedule(intervalMs: Long, timestamp: Float) = Unit
            }
        """.trimIndent()

        assertEquals(6, rule.lint(code).size)
    }

    @Test
    fun GIVEN_time_named_values_with_time_types_WHEN_linted_THEN_reports_nothing() {
        val code = """
            import kotlin.time.Duration
            import kotlin.time.Instant

            class Session(val createdAt: Instant, val timeout: Duration?) {
                val delay = 5
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_primitives_with_unrelated_names_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class Page(val format: String, val flat: Int, val runtime: Long, val items: List<Long>) {
                fun read(handler: (Int) -> Unit) = Unit
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_serializable_class_WHEN_linted_THEN_reports_nothing() {
        val code = """
            import kotlinx.serialization.Serializable

            @Serializable
            class SessionDto(val createdAt: Long, val timeoutMs: Int)
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_override_WHEN_linted_THEN_reports_only_the_overridden_declaration() {
        val code = """
            interface Scheduler {
                fun schedule(delayMs: Long)
            }

            class TimerScheduler(override val startedAt: Long) : Scheduler {
                override fun schedule(delayMs: Long) = Unit
            }
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_top_level_time_primitive_WHEN_linted_THEN_reports_it() {
        val code = """
            val startedAt: Long = 0L
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }
}
