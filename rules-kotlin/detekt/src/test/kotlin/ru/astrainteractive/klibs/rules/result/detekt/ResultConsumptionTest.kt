@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.result.detekt

import dev.detekt.api.Config
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.createEnvironment
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ResultConsumptionTest {
    private val rule = ResultConsumption(Config.empty)
    private val environment = createEnvironment()

    @Test
    fun GIVEN_result_in_an_assigned_if_WHEN_linted_THEN_keeps_both_branch_values() {
        val code = """
            fun load(exists: Boolean): Result<Int> {
                val result = if (exists) {
                    runCatching { 1 }
                } else {
                    runCatching { 2 }
                }
                return result.onSuccess { value -> println(value) }
            }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_result_in_returned_when_and_try_WHEN_linted_THEN_keeps_branch_values() {
        val code = """
            fun load(mode: Int): Result<Int> {
                return when (mode) {
                    0 -> { runCatching { 1 } }
                    else -> {
                        try {
                            runCatching { 2 }
                        } catch (error: Exception) {
                            Result.failure(error)
                        }
                    }
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_nested_branch_in_value_lambda_WHEN_linted_THEN_keeps_the_result() {
        val code = """
            fun load(enabled: Boolean): () -> Result<Int> = {
                if (enabled) {
                    when {
                        enabled -> { runCatching { 1 } }
                        else -> { runCatching { 2 } }
                    }
                } else {
                    runCatching { 3 }
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_discarded_branch_values_WHEN_linted_THEN_reports_each_result() {
        val code = """
            fun load(enabled: Boolean) {
                if (enabled) { runCatching { 1 } } else { runCatching { 2 } }
                when (enabled) {
                    true -> { runCatching { 3 } }
                    false -> { runCatching { 4 } }
                }
                try { runCatching { 5 } } catch (error: Exception) { runCatching { 6 } }
            }
        """.trimIndent()

        assertEquals(6, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_branch_values_in_unit_lambda_WHEN_linted_THEN_reports_discarded_results() {
        val code = """
            fun load(values: List<Boolean>) {
                values.forEach { enabled ->
                    if (enabled) { runCatching { 1 } } else { runCatching { 2 } }
                }
            }
        """.trimIndent()

        assertEquals(2, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_unused_statement_and_finally_in_used_try_WHEN_linted_THEN_reports_only_discarded_results() {
        val code = """
            fun load(): Result<Int> = try {
                runCatching { 1 }
                runCatching { 2 }
            } finally {
                runCatching { 3 }
            }
        """.trimIndent()

        assertEquals(2, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_discarded_results_WHEN_linted_THEN_reports_each_statement() {
        val code = """
            fun consume() {
                runCatching { 1 }
                runCatching { 2 }.map { value -> value + 1 }
            }
        """.trimIndent()

        assertEquals(2, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_consumed_results_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun consume(): Int {
                val assigned = runCatching { 1 }
                runCatching { 2 }.onSuccess { value -> println(value) }
                runCatching { 3 }.onFailure { error -> println(error) }
                runCatching { 4 }.fold(
                    onSuccess = { value -> println(value) },
                    onFailure = { error -> println(error) }
                )
                runCatching { 5 }.getOrElse { 0 }
                listOf(1).map { value -> value + 1 }
                println(runCatching { 6 })
                return assigned.getOrThrow()
            }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_result_as_the_last_expression_of_lambdas_WHEN_linted_THEN_reports_only_the_unit_lambda() {
        val code = """
            inline fun <T> withLock(block: () -> T): T = block()

            fun save(): Result<Int> {
                return withLock {
                    runCatching { 1 }
                }
            }

            fun saveAll(ids: List<Int>) {
                ids.forEach { id ->
                    runCatching { id }
                }
            }
        """.trimIndent()

        assertEquals(1, rule.lintWithContext(environment, code).size)
    }
}
