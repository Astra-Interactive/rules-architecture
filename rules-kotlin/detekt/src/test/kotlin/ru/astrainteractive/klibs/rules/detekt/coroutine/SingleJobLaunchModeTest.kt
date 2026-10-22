@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.coroutine

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class SingleJobLaunchModeTest {
    private val rule = SingleJobLaunchMode(Config.empty)

    @Test
    fun GIVEN_unguarded_launches_WHEN_linted_THEN_reports_each() {
        val code = """
            class PaywallViewModel {
                private val purchaseScope = viewModelScope.asSingleJobScope()

                fun onPurchase() {
                    purchaseScope.launch { purchase() }
                    viewModelScope.launch { purchase() }
                }
            }

            class RatingComponent(private val scope: CoroutineScope) {
                fun onRate() {
                    scope.launch { rate() }
                }
            }
        """.trimIndent()

        val messages = rule.lint(code).map { finding -> finding.message }

        assertEquals(
            listOf(
                "Pass the SingleJobMode to 'purchaseScope.launch': skip while running for a purchase, cancel the " +
                    "previous for a search; never leave it to the default.",
                "Guard this launch against a second click: launch on a single-job scope " +
                    "('viewModelScope.asSingleJobScope()' with a SingleJobMode), or claim the state with " +
                    "getAndUpdate first.",
                "Guard this launch against a second click: launch on a single-job scope " +
                    "('scope.asSingleJobScope()' with a SingleJobMode), or claim the state with getAndUpdate first.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_guarded_or_unrelated_launches_WHEN_linted_THEN_reports_nothing() {
        val code = """
            private val topScope = createScope()

            class PaywallViewModel {
                private val purchaseScope = viewModelScope.asSingleJobScope()
                private val plainScope = viewModelScope.plus(job)
                private val otherScope = viewModelScope.child

                init {
                    viewModelScope.launch { load() }
                }

                fun onPurchase() {
                    purchaseScope.launch(SingleJobMode.SKIP_IF_RUNNING) { purchase() }
                    plainScope.launch { purchase() }
                    otherScope.launch { purchase() }
                    topScope.launch { purchase() }
                    holder.scope.launch { purchase() }
                    launch { purchase() }
                    this.viewModelScope.launch { purchase() }
                }

                fun refresh() {
                    viewModelScope.launch { load() }
                }
            }

            class Loader(private val scope: CoroutineScope) {
                fun onStart() {
                    scope.launch { load() }
                }
            }

            fun onClick(scope: CoroutineScope) {
                scope.launch { load() }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
