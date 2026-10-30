@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.result.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class RunCatchingCancellationTest {
    private val rule = RunCatchingCancellation(Config.empty)

    @Test
    fun GIVEN_run_catching_in_coroutine_code_without_propagation_WHEN_linted_THEN_reports_each() {
        val code = """
            suspend fun load(client: Client): Result<Int> {
                return runCatching { client.fetch() }
            }

            class Loader(private val client: Client) {
                fun start() {
                    scope.launch {
                        listOf(1).map { client.runCatching { fetch() }.getOrNull() }
                    }
                }
            }

            fun test() = runTest {
                runCatching { 1 }.propagateCancellationException().also { runCatching { 2 } }
            }

            suspend fun loadWithOtherChecks(client: Client) {
                runCatching { client.fetch() }.onFailure(::log)
                runCatching { client.fetch() }.onFailure { error -> log(error) }
                runCatching { client.fetch() }.onFailure { error -> if (error.isFatal) throw error }
                runCatching { client.fetch() }.onFailure { error -> if (error !is CancellationException) throw error }
                runCatching { client.fetch() }.onFailure { error -> if (error is IOException) throw error }
                runCatching { client.fetch() }.onFailure { error -> if (error is CancellationException) log(error) }
                runCatching { client.fetch() }.map { value -> value }
            }
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(10, messages.size)
        assertEquals(
            "runCatching in coroutine code catches CancellationException: call .propagateCancellationException() " +
                "right after it, or without mikro .onFailure { error -> if (error is CancellationException) throw " +
                "error }.",
            messages.first()
        )
    }

    @Test
    fun GIVEN_propagated_or_blocking_run_catching_WHEN_linted_THEN_reports_nothing() {
        val code = """
            suspend fun load(client: Client): Result<Int> {
                return client.runCatching { fetch() }
                    .propagateCancellationException()
                    .map { value -> value + 1 }
            }

            fun parse(raw: String): Result<Int> = runCatching { raw.toInt() }

            fun parseAll(raw: List<String>) = raw.map { value -> runCatching { value.toInt() }.getOrNull() }

            class Parser {
                val parsed = runCatching { 1 }
            }

            val topLevel = runCatching { 1 }

            suspend fun read() = listOf(1).map { value -> value }.let { values -> values.size }

            suspend fun loadInline(client: Client): Result<Int> {
                return runCatching { client.fetch() }
                    .onFailure { error -> if (error is kotlinx.coroutines.CancellationException) throw error }
            }

            suspend fun loadInlineBlock(client: Client): Result<Int> {
                return runCatching { client.fetch() }
                    .onFailure { error ->
                        if (error is CancellationException) {
                            throw error
                        }
                    }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
