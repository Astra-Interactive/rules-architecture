@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.createEnvironment
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class HardcodedDispatcherTest {
    private val rule = HardcodedDispatcher(Config.empty)
    private val environment = createEnvironment()

    @Test
    fun GIVEN_dispatchers_read_from_kotlinx_WHEN_linted_THEN_reports_each() {
        val code = """
            package com.example.rating.command

            import kotlinx.coroutines.Dispatchers
            import kotlinx.coroutines.Dispatchers.Unconfined

            class RatingCommandExecutor {
                val io = Dispatchers.IO
                val default = Dispatchers.Default.limitedParallelism(2)
                val unconfined = Unconfined
                val main = Dispatchers.Main
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Take the IO dispatcher from the injected dispatchers instead of Dispatchers.IO: a class gets its " +
                    "dispatchers through the constructor, so a test can replace them.",
                "Take the Default dispatcher from the injected dispatchers instead of Dispatchers.Default: a class " +
                    "gets its dispatchers through the constructor, so a test can replace them.",
                "Take the Unconfined dispatcher from the injected dispatchers instead of Dispatchers.Unconfined: a " +
                    "class gets its dispatchers through the constructor, so a test can replace them.",
            ),
            rule.lintWithContext(
                environment,
                code,
                COROUTINES
            ).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_injected_dispatchers_providers_and_di_WHEN_linted_THEN_reports_nothing() {
        val findings = listOf(
            INJECTED,
            PROVIDER,
            DI_MODULE
        ).flatMap { code ->
            rule.lintWithContext(
                environment,
                code,
                COROUTINES,
                DISPATCHERS_INTERFACES
            )
        }

        assertEquals(emptyList(), findings.map(Finding::message))
    }

    private companion object {
        val INJECTED = """
            package com.example.rating.command

            import com.example.core.coroutine.KotlinDispatchers
            import com.example.core.coroutine.AppDispatchers

            class RatingCommandExecutor(
                private val dispatchers: KotlinDispatchers,
                private val appDispatchers: AppDispatchers
            ) {
                val io = dispatchers.IO
                val unconfined = dispatchers.Unconfined
                val custom = appDispatchers.IO

                fun read(other: KotlinDispatchers) = other.Default
            }
        """.trimIndent()

        val PROVIDER = """
            package com.example.core.coroutine

            import kotlinx.coroutines.CoroutineDispatcher
            import kotlinx.coroutines.Dispatchers

            interface AppDispatchers {
                val IO: CoroutineDispatcher
            }

            object DefaultAppDispatchers : AppDispatchers {
                override val IO = Dispatchers.IO
            }

            class GetterAppDispatchers : AppDispatchers {
                override val IO: CoroutineDispatcher
                    get() = Dispatchers.IO
            }
        """.trimIndent()

        val DI_MODULE = """
            package com.example.core.di

            import kotlinx.coroutines.CoroutineScope
            import kotlinx.coroutines.Dispatchers

            class CoreModule {
                val ioScope = CoroutineScope(Dispatchers.IO)
            }
        """.trimIndent()

        val COROUTINES = """
            package kotlinx.coroutines

            abstract class CoroutineDispatcher {
                fun limitedParallelism(parallelism: Int): CoroutineDispatcher = this
            }

            class CoroutineScope(val dispatcher: CoroutineDispatcher)

            object Dispatchers {
                val IO: CoroutineDispatcher = TODO()
                val Default: CoroutineDispatcher = TODO()
                val Unconfined: CoroutineDispatcher = TODO()
                val Main: CoroutineDispatcher = TODO()
            }
        """.trimIndent()

        val DISPATCHERS_INTERFACES = """
            package com.example.core.coroutine

            import kotlinx.coroutines.CoroutineDispatcher

            interface KotlinDispatchers {
                val IO: CoroutineDispatcher
                val Default: CoroutineDispatcher
                val Unconfined: CoroutineDispatcher
            }
        """.trimIndent()
    }
}
