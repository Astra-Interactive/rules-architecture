@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.injection.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class RootModuleConstructionTest {
    private val rule = RootModuleConstruction(Config.empty)

    @Test
    fun GIVEN_root_module_constructing_services_WHEN_linted_THEN_reports_each() {
        val code = """
            package x.di

            import x.core.di.CoreModule
            import x.coroutine.DefaultDispatchers
            import x.other.Other
            import x.rating.MisplacedModule
            import x.rating.exposed.ExposedRatingRepository
            import x.rating.file.FileRatingRepository
            import x.rating.memory.InMemoryRatingRepository

            class RootModule(plugin: Plugin) {
                val coreModule = CoreModule(dispatchers = DefaultDispatchers())
                val repository by lazy { ExposedRatingRepository(coreModule.database) }
                val other = Other.Lambda()
                val misplaced = MisplacedModule()

                init {
                    val initRepository = InMemoryRatingRepository()
                }

                fun configure() {
                    FileRatingRepository()
                }
            }
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(6, messages.size)
        assertEquals(
            "RootModule constructs 'DefaultDispatchers': construct it in the DI module that owns it and pass that " +
                "module; RootModule only builds the root DI modules.",
            messages.first()
        )
    }

    @Test
    fun GIVEN_root_module_building_modules_of_di_packages_and_lifecycles_WHEN_linted_THEN_reports_nothing() {
        val code = """
            package x.root.di

            import x.core.di.CoreModule
            import x.jail.di.JailModule
            import x.theme.di.ThemeSwitcherModuleImpl
            import x.status.di.StatusModule as Status

            interface RootModule {
                class Default : RootModule {
                    val coreModule = CoreModule.Default()
                    val jailModule by lazy { JailModule(coreModule) }
                    val themeModule = ThemeSwitcherModuleImpl(coreModule = coreModule)
                    val statusModule = Status(coreModule)
                    val apiModule = x.api.di.ApiModule.Default(coreModule)
                    val pagerModule = PagerModule(coreModule)
                    val lifecycle = Lifecycle.Lambda(onEnable = { listOf(coreModule.lifecycle).forEach(::enable) })
                    val composite = CompositeLifecycle()
                    val name = "root"
                }
            }

            class OtherModule {
                val repository = ExposedRatingRepository()
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lint(code).map(Finding::message))
    }
}
