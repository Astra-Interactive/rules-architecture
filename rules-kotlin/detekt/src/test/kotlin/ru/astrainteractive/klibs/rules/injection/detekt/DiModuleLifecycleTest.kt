@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.injection.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
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

        val messages = rule.lint(code).map(Finding::message)

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

    @Test
    fun GIVEN_modules_that_start_something_without_a_lifecycle_WHEN_linted_THEN_reports_each() {
        val code = """
            class RatingStorageModule(scope: CoroutineScope) {
                val warmUp = RatingCacheWarmUp(scope)
            }

            class EventModule {
                val killListener = KillEventListener()
            }

            class TimerModule(scope: CoroutineScope) {
                val job = scope.launch { }
            }

            class GuiModule {
                val router = BukkitGuiRouter()
            }

            class JailModule {
                val listener = JailListener()
                val lifecycle = Lifecycle.Lambda(onEnable = { listener.register() })
            }
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(
            listOf(
                "Expose val lifecycle: Lifecycle from RatingStorageModule: it starts RatingCacheWarmUp, and the " +
                    "parent module starts and stops it through that lifecycle.",
                "Expose val lifecycle: Lifecycle from EventModule: it starts KillEventListener, and the parent " +
                    "module starts and stops it through that lifecycle.",
                "Expose val lifecycle: Lifecycle from TimerModule: it starts launch, and the parent module starts " +
                    "and stops it through that lifecycle.",
            ),
            messages
        )
    }
}
