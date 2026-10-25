@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.time.detekt

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
                val delay = 5.seconds
                val createdAtFlag = true
                val onTimeout: () -> Unit = {}
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

    @Test
    fun GIVEN_upper_snake_time_names_typed_or_inferred_as_primitives_WHEN_linted_THEN_reports_each() {
        val code = """
            private const val CARD_IN_MS = 400
            private const val RETRY_DELAY: Long = 3L
            private const val TIMEOUT_SECONDS = 1.5
            private const val CREATED_AT = "2024-01-01"
            private const val FADE_MILLIS = 0x10
            private const val TIMESTAMP = 2f
        """.trimIndent()

        assertEquals(6, rule.lint(code).size)
    }

    @Test
    fun GIVEN_upper_snake_names_without_a_time_word_or_with_duration_values_WHEN_linted_THEN_reports_nothing() {
        val code = """
            private const val MAX_ITEMS = 100
            private const val CARD_IN = 400
            private const val SECONDS = 60
            private const val LOOK_AT_ME = "x"
            private val CardIn = 400.milliseconds
            private val IS_TIMEOUT = true
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
