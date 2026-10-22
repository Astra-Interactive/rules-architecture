@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.injection

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class RootModuleConstructionTest {
    private val rule = RootModuleConstruction(Config.empty)

    @Test
    fun GIVEN_root_module_constructing_services_WHEN_linted_THEN_reports_each() {
        val code = """
            class RootModule(plugin: Plugin) {
                val coreModule = CoreModule(dispatchers = DefaultDispatchers())
                val repository by lazy { ExposedRatingRepository(coreModule.database) }
                val other = Other.Lambda()
            }
        """.trimIndent()

        val messages = rule.lint(code).map { finding -> finding.message }

        assertEquals(3, messages.size)
        assertEquals(
            "RootModule constructs 'DefaultDispatchers': construct it in the DI module that owns it and pass that " +
                "module; RootModule only builds the root DI modules.",
            messages.first()
        )
    }

    @Test
    fun GIVEN_root_module_building_modules_and_lifecycles_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class RootModule(plugin: Plugin) {
                val coreModule = CoreModule(plugin)
                val jailModule by lazy { JailModule(coreModule) }
                val lifecycle = Lifecycle.Lambda(onEnable = { listOf(coreModule.lifecycle).forEach(::enable) })
                val composite = CompositeLifecycle()
                val name = "root"
            }

            class OtherModule {
                val repository = ExposedRatingRepository()
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
