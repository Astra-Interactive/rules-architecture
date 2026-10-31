@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.result.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.createEnvironment
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class TryCatchCancellationTest {
    private val rule = TryCatchCancellation(Config.empty)
    private val environment = createEnvironment()

    @Test
    fun GIVEN_catch_swallowing_cancellation_in_suspend_code_WHEN_linted_THEN_reports_it() {
        val code = """
            suspend fun fetch(): Int = 1

            suspend fun load(): Int {
                return try {
                    fetch()
                } catch (error: Exception) {
                    0
                }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Rethrow CancellationException before handling the failure: this catch in suspend code also " +
                    "catches the cancellation and breaks structured concurrency; catch the exception you expect " +
                    "instead."
            ),
            rule.lintWithContext(environment, code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_documented_run_catching_and_safe_catches_WHEN_linted_THEN_reports_nothing() {
        val code = """
            import java.io.IOException
            import kotlin.coroutines.cancellation.CancellationException

            suspend fun fetch(): Int = 1

            fun <T> Result<T>.propagateCancellationException(): Result<T> = this

            suspend fun loadResult(): Result<Int> {
                return runCatching { fetch() }.propagateCancellationException()
            }

            suspend fun loadRethrown(): Int {
                return try {
                    fetch()
                } catch (cancellation: CancellationException) {
                    throw cancellation
                } catch (error: Exception) {
                    0
                }
            }

            suspend fun loadSpecific(): Int {
                return try {
                    fetch()
                } catch (error: IOException) {
                    0
                }
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lintWithContext(environment, code).map(Finding::message))
    }
}
