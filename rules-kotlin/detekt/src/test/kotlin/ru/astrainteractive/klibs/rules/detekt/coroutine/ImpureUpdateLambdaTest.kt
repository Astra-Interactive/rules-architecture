@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.coroutine

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ImpureUpdateLambdaTest {
    private val rule = ImpureUpdateLambda(Config.empty)

    @Test
    fun GIVEN_update_lambdas_assigning_outer_names_WHEN_linted_THEN_reports_each() {
        val code = """
            class SplashViewModel {
                private var count = 0
                private val values = IntArray(1)

                fun start(): Int? {
                    var started: Int? = null
                    state.update { current ->
                        current.copy(isBusy = true).also { busy -> started = busy }
                    }
                    state.getAndUpdate({ current -> this.count += 1; current })
                    state.updateAndGet { current ->
                        values[0] = 1
                        count++
                        holder.total = 2
                        current
                    }
                    return started
                }
            }
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(5, messages.size)
        assertEquals(
            "The update lambda assigns 'started', declared outside it: the lambda may run twice, so keep it pure " +
                "and use getAndUpdate with a pure transition instead.",
            messages.first()
        )
    }

    @Test
    fun GIVEN_pure_update_lambdas_and_other_calls_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class SplashViewModel {
                private var count = 0

                fun start() {
                    state.update { current ->
                        var copy = current
                        copy = copy.next()
                        val (first, second) = copy
                        var index = first
                        index++
                        (getValue())++
                        listOf(1).forEach { item -> var local = item; local += second }
                        copy
                    }
                    state.update(::transition)
                    state.update { current -> current.list[0] }
                    apply { count = 1 }
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
