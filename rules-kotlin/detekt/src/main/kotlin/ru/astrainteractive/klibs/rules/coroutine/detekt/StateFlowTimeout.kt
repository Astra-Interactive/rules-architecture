package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.RequiresAnalysisApi
import dev.detekt.api.Rule
import org.jetbrains.kotlin.analysis.api.analyze
import org.jetbrains.kotlin.analysis.api.types.symbol
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtQualifiedExpression
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports `StateFlow.first()` inside `withTimeout` or `withTimeoutOrNull`: the current value never waits, so the
 * timeout cannot fire. A predicate or a SharedFlow read may actually wait and is allowed.
 */
internal class StateFlowTimeout(config: Config) : Rule(config, DESCRIPTION), RequiresAnalysisApi {
    private fun isStateFlowFirst(call: KtCallExpression): Boolean {
        if (call.calleeExpression?.text != FIRST || call.valueArguments.isNotEmpty()) return false
        if (call.lambdaArguments.isNotEmpty()) return false
        val qualified = call.parent as? KtQualifiedExpression ?: return false
        if (qualified.selectorExpression != call) return false
        val receiver = qualified.receiverExpression
        return analyze(receiver) {
            receiver.expressionType?.symbol?.classId?.asFqNameString() in STATE_FLOW_TYPES
        }
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        if (expression.calleeExpression?.text !in TIMEOUT_CALLS) return
        expression.collectDescendantsOfType<KtCallExpression>()
            .filter(::isStateFlowFirst)
            .forEach { call ->
                val message = "Read StateFlow with .value, not first() inside a timeout: the current value cannot " +
                    "wait, so the timeout never fires."
                report(Finding(Entity.from(call), message))
            }
    }

    private companion object {
        const val FIRST = "first"

        @JvmField
        val TIMEOUT_CALLS = setOf("withTimeout", "withTimeoutOrNull")

        @JvmField
        val STATE_FLOW_TYPES = setOf(
            "kotlinx.coroutines.flow.MutableStateFlow",
            "kotlinx.coroutines.flow.StateFlow"
        )
        const val DESCRIPTION = "A StateFlow is never read with first() inside a timeout."
    }
}
