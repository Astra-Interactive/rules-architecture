@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.result

import dev.detekt.api.Config
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.createEnvironment
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ResultConsumptionTest {
    private val rule = ResultConsumption(Config.empty)
    private val environment = createEnvironment()

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
}
