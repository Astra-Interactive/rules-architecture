@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.injection.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.core.config.YamlConfig
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.createEnvironment
import org.junit.jupiter.api.Test
import java.io.StringReader
import kotlin.test.assertEquals

internal class DiModuleLifecycleTest {
    private val rule = DiModuleLifecycle(Config.empty)
    private val environment = createEnvironment()

    @Test
    fun GIVEN_modules_with_own_lifecycle_functions_WHEN_linted_THEN_reports_each() {
        val code = """
            import ru.astrainteractive.astralibs.lifecycle.Lifecycle
            class JailModule {
                fun enable() = Unit
                fun onReload() = Unit
            }

            interface CoreModule {
                fun disable()
            }
        """.trimIndent()

        val messages = rule.lintWithContext(
            environment,
            code,
            LIFECYCLE_SOURCE
        ).map(Finding::message)

        assertEquals(3, messages.size)
        assertEquals(
            "Remove 'enable' from JailModule: expose val lifecycle: Lifecycle and let the parent module collect it.",
            messages.first()
        )
    }

    @Test
    fun GIVEN_lifecycle_property_and_other_classes_WHEN_linted_THEN_reports_nothing() {
        val code = """
            import ru.astrainteractive.astralibs.lifecycle.Lifecycle

            class JailModule {
                val lifecycle = Lifecycle.Lambda(onEnable = { start() })

                private fun start() = Unit
            }

            class JailPlugin {
                fun onEnable() = Unit
            }

            fun onEnable() = Unit
        """.trimIndent()

        assertEquals(
            0,
            rule.lintWithContext(
                environment,
                code,
                LIFECYCLE_SOURCE
            ).size
        )
    }

    @Test
    fun GIVEN_modules_that_start_something_without_a_lifecycle_WHEN_linted_THEN_reports_each() {
        val code = """
            import ru.astrainteractive.astralibs.lifecycle.Lifecycle
            import kotlinx.coroutines.CoroutineScope
            import kotlinx.coroutines.launch

            class RatingCacheWarmUp(scope: CoroutineScope) { val lifecycle = Lifecycle() }
            class KillEventListener { val lifecycle = Lifecycle() }
            class BukkitGuiRouter
            class JailListener { fun register() = Unit }

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

        val messages = rule.lintWithContext(
            environment,
            code,
            LIFECYCLE_SOURCE
        ).map(Finding::message)

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

    @Test
    fun GIVEN_owned_lazy_and_injected_modules_WHEN_lifecycles_collected_THEN_reports_borrowed_instances_only() {
        val code = """
            package x.di

            import ru.astrainteractive.astralibs.lifecycle.Lifecycle

            class Child { val lifecycle = Lifecycle() }
            fun createChild(): Child = Child()

            class CompositionModule(private val injected: Child) {
                private val alias = injected
                private val owned = Child()
                private val deferred by lazy { Child() }
                private val unknown = createChild()
                val lifecycle = Lifecycle.merge(
                    owned.lifecycle,
                    deferred.lifecycle,
                    unknown.lifecycle,
                    injected.lifecycle,
                    alias.lifecycle
                )
            }

            class ConsumerModule(private val child: Child) {
                fun stop() { child.lifecycle.onDisable() }
                fun inspect(): Lifecycle = child.lifecycle
            }
        """.trimIndent()
        val messages = rule.lintWithContext(
            environment,
            code,
            LIFECYCLE_SOURCE
        ).map(Finding::message)

        assertEquals(3, messages.size)
        assertEquals(
            listOf(
                "injected.lifecycle",
                "alias.lifecycle",
                "child.lifecycle"
            ),
            messages.map { message -> message.substringAfter("'").substringBefore("'") }
        )
    }

    @Test
    fun GIVEN_service_names_injected_aliases_and_factories_WHEN_checked_THEN_does_not_invent_resource_ownership() {
        val code = """
            import ru.astrainteractive.astralibs.lifecycle.Lifecycle
            class Child { val lifecycle = Lifecycle() }
            class PricingService
            fun createChild(): Child = Child()

            class ConsumerModule(injected: Child) {
                val dependency = injected
                val alias = dependency
                val unknown = createChild()
                val plain = PricingService()
                fun launch() = Unit
                val value = launch()
            }

            class DeferredModule {
                val child by lazy { Child() }
            }

            class Resource : AutoCloseable { override fun close() = Unit }
            class ResourceModule { val resource = Resource() }
        """.trimIndent()
        val messages = rule.lintWithContext(
            environment,
            code,
            LIFECYCLE_SOURCE
        ).map(Finding::message)

        assertEquals(2, messages.size)
        assertEquals(
            listOf("DeferredModule", "ResourceModule"),
            messages.map { message -> message.substringAfter("from ").substringBefore(":") }
        )
    }

    @Test
    fun GIVEN_app_modules_without_minecraft_convention_WHEN_checked_THEN_does_not_require_lifecycle() {
        val code = """
            import kotlinx.coroutines.CoroutineScope
            import kotlinx.coroutines.launch

            class HttpClient : AutoCloseable { override fun close() = Unit }
            interface CoreModule {
                class Default(scope: CoroutineScope) : CoreModule {
                    val client = HttpClient()
                    val job = scope.launch { }
                }
            }
            class RootModule {
                fun enable() = Unit
            }
        """.trimIndent()

        assertEquals(
            0,
            rule.lintWithContext(
                environment,
                code,
                LIFECYCLE_SOURCE
            ).size
        )
    }

    @Test
    fun GIVEN_lifecycle_implementation_in_di_WHEN_checked_THEN_preserves_its_behavior() {
        val code = """
            package x.di

            import ru.astrainteractive.astralibs.lifecycle.Lifecycle

            class FeatureGate : Lifecycle() {
                override fun onEnable() = Unit
                override fun onDisable() = Unit
            }
            class PreviewProjectService {
                val rootModule = RootModule()
                fun onEnable() = rootModule.lifecycle.onEnable()
            }
            class RootModule { val lifecycle = Lifecycle() }
        """.trimIndent()

        assertEquals(
            0,
            rule.lintWithContext(
                environment,
                code,
                LIFECYCLE_SOURCE
            ).size
        )
    }

    @Test
    fun GIVEN_unrelated_lifecycle_type_WHEN_resources_owned_THEN_does_not_apply_minecraft_convention() {
        val code = """
            package example.di

            class Lifecycle
            class Child { val lifecycle = Lifecycle() }
            class FeatureModule {
                val child = Child()
                fun onEnable() = Unit
            }
        """.trimIndent()

        assertEquals(
            0,
            rule.lintWithContext(
                environment,
                code,
                LIFECYCLE_SOURCE
            ).size
        )
    }

    @Test
    fun GIVEN_owned_astralibs_resource_without_import_WHEN_checked_THEN_requires_module_lifecycle() {
        val code = """
            interface CoreModule {
                class Default : CoreModule {
                    val child = Child()
                }
            }
            class Child {
                val lifecycle = ru.astrainteractive.astralibs.lifecycle.api.Lifecycle()
            }
        """.trimIndent()
        val lifecycle = LIFECYCLE_SOURCE.replace(
            "package ru.astrainteractive.astralibs.lifecycle",
            "package ru.astrainteractive.astralibs.lifecycle.api"
        )
        val messages = rule.lintWithContext(
            environment,
            code,
            lifecycle
        ).map(Finding::message)

        assertEquals(1, messages.size)
        assertEquals(
            "Expose val lifecycle: Lifecycle from Default: it starts Child, and the parent module " +
                "starts and stops it through that lifecycle.",
            messages.single()
        )
    }

    @Test
    fun GIVEN_custom_lifecycle_convention_WHEN_configured_THEN_checks_modules_but_not_lifecycle_implementations() {
        val code = """
            import custom.Lifecycle
            class Child { val lifecycle = Lifecycle() }
            class ChildModule { val child = Child() }
            class GateModule : Lifecycle() { override fun onEnable() = Unit }
        """.trimIndent()
        val lifecycle = "package custom\nopen class Lifecycle { open fun onEnable() = Unit }"
        val config = YamlConfig.load(StringReader("lifecycleTypes: [ custom.Lifecycle ]"))

        assertEquals(
            0,
            rule.lintWithContext(
                environment,
                code,
                lifecycle
            ).size
        )
        assertEquals(
            1,
            DiModuleLifecycle(config).lintWithContext(
                environment,
                code,
                lifecycle
            ).size
        )
    }

    private companion object {
        val LIFECYCLE_SOURCE = """
            package ru.astrainteractive.astralibs.lifecycle

            open class Lifecycle {
                open fun onEnable() = Unit
                open fun onDisable() = Unit
                class Lambda(val onEnable: () -> Unit) : Lifecycle()
                companion object { fun merge(vararg children: Lifecycle): Lifecycle = Lifecycle() }
            }
        """.trimIndent()
    }
}
