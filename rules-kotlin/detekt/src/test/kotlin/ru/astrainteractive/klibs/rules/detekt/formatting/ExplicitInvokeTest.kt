@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.formatting

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ExplicitInvokeTest {
    private val rule = ExplicitInvoke(Config.empty)

    @Test
    fun GIVEN_values_called_without_invoke_WHEN_linted_THEN_reports_each_call() {
        val code = """
            class Splash(private val provider: () -> Int, onClose: () -> Unit) {
                private val factory: () -> String = { "" }
                private val close = onClose

                init {
                    onClose()
                }

                fun load(useCase: (Int) -> Int): Int {
                    val mapper: (Int) -> Int = { value -> value }
                    factory()
                    return listOf(1).map { transform: (Int) -> Int -> transform(useCase(mapper(provider()))) }.first()
                }
            }

            object Holder {
                val action: () -> Unit = {}

                fun run() = action()
            }
        """.trimIndent()

        val findings = rule.lint(code)

        assertEquals(7, findings.size)
        assertEquals(
            "'onClose' is a value: call it as 'onClose.invoke(…)' so the reader sees a value being called.",
            findings.first().message
        )
    }

    @Test
    fun GIVEN_explicit_invoke_functions_and_receivers_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class Splash(private val provider: () -> Int, private val load: () -> Int) {
                private fun load(): Int = 1

                fun read(holder: Holder): Int {
                    val later = provider.invoke()
                    val other: () -> Int = { later }
                    holder.other()
                    this.provider()
                    unknown()
                    return load() + other.invoke()
                }
            }

            fun before(): Int {
                val value = compute()
                val compute: () -> Int = { value }
                return compute.invoke()
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
