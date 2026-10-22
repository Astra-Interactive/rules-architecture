@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.injection

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class DiModuleLifecycleTest {
    private val rule = DiModuleLifecycle(Config.empty)

    @Test
    fun GIVEN_modules_with_own_lifecycle_functions_WHEN_linted_THEN_reports_each() {
        val code = """
            class JailModule {
                fun enable() = Unit
                fun onReload() = Unit
            }

            interface CoreModule {
                fun disable()
            }
        """.trimIndent()

        val messages = rule.lint(code).map { finding -> finding.message }

        assertEquals(3, messages.size)
        assertEquals(
            "Remove 'enable' from JailModule: expose val lifecycle: Lifecycle and let the parent module collect it.",
            messages.first()
        )
    }

    @Test
    fun GIVEN_lifecycle_property_and_other_classes_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class JailModule {
                val lifecycle = Lifecycle.Lambda(onEnable = { start() })

                private fun start() = Unit
            }

            class JailPlugin {
                fun onEnable() = Unit
            }

            fun onEnable() = Unit
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
