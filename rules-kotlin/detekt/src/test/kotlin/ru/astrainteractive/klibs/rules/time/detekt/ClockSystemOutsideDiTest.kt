@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.time.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ClockSystemOutsideDiTest {
    private val rule = ClockSystemOutsideDi(Config.empty)

    @Test
    fun GIVEN_clock_system_read_in_a_class_WHEN_linted_THEN_reports_each_read() {
        val code = """
            package x.jail.jail.usecase

            class FreeInmateUseCase {
                fun invoke() = Clock.System.now()

                fun later() = kotlin.time.Clock.System.now()
            }
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(2, messages.size)
        assertEquals(
            "Take a Clock through the constructor instead of reading Clock.System here: the DI module passes " +
                "Clock.System.",
            messages.first()
        )
    }

    @Test
    fun GIVEN_clock_system_passed_by_a_di_module_or_an_injected_clock_WHEN_linted_THEN_reports_nothing() {
        val code = """
            package x.jail.di

            class JailModule {
                val freeInmateUseCase = FreeInmateUseCase(Clock.System)
            }

            class FreeInmateUseCase(private val clock: Clock) {
                fun invoke() = clock.now()
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lint(code).map(Finding::message))
    }
}
