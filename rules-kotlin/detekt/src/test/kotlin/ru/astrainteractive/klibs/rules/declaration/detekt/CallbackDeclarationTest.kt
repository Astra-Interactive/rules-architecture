@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.declaration.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import dev.detekt.test.utils.compileForTest
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.io.File
import kotlin.test.assertEquals

internal class CallbackDeclarationTest {
    private val rule = CallbackDeclaration(Config.empty)

    @Test
    fun GIVEN_callbacks_in_project_api_WHEN_linted_THEN_reports_each() {
        val code = """
            interface PermissionGateway {
                fun requestPermission(onResult: (Boolean) -> Unit)
                fun loadOffer(onSuccess: ((Offer) -> Unit)?, onFailure: (Throwable) -> Unit)
            }

            interface TimerListener {
                suspend fun onStarted(config: TimerConfig)
            }

            interface PurchaseCallback

            class CheckoutComponent(private val onOutput: (CheckoutOutput) -> Unit) {
                val onClose: () -> Unit = {}
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "onResult",
                "onSuccess",
                "onFailure",
                "TimerListener",
                "PurchaseCallback",
                "onOutput",
                "onClose"
            ),
            rule.lint(code).map(Finding::message).map { message -> NAME.find(message)?.groupValues?.get(1) }
        )
    }

    @Test
    fun GIVEN_composable_handlers_outputs_and_values_WHEN_linted_THEN_reports_nothing(@TempDir directory: File) {
        val code = """
            @Composable
            fun PaywallComposable(onClick: () -> Unit, onValueChange: (String) -> Unit) = Unit

            fun Modifier.revealGesture(onReveal: () -> Unit): Modifier = this

            fun interface OnPaidCallback {
                operator fun invoke(order: OrderId)
            }

            class CheckoutComponent(private val onPaidCallback: OnPaidCallback, private val onlineCount: Int)

            class Reloader(private val block: suspend () -> Unit) : Gateway {
                override val onReady: () -> Unit = {}

                fun reload() {
                    val onDone: () -> Unit = {}
                    listOf(1).forEach { onItem -> println(onItem) }
                }
            }

            class Plain(val onLabel: String)
        """.trimIndent()
        val testFile = File(directory, "src/test/kotlin/x/FakeGateway.kt")
        testFile.parentFile.mkdirs()
        testFile.writeText(TEST_SOURCE)

        assertEquals(0, rule.lint(code).size + rule.lint(compileForTest(testFile.toPath())).size)
    }

    private companion object {
        val NAME = Regex("""'(\w+)'""")

        val TEST_SOURCE = """
            interface FakeListener

            fun load(onResult: (Int) -> Unit) = Unit
        """.trimIndent()
    }
}
