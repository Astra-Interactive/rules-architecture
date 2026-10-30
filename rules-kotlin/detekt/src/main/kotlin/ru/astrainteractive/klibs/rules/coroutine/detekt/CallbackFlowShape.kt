package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtLambdaExpression
import org.jetbrains.kotlin.psi.KtValueArgument
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports a callback bridge out of the no-callbacks shape: a `callbackFlow` without `awaitClose`, a `launch { send }`
 * inside it, which loses the order and starts a coroutine per event, and a `buffer(Channel.RENDEZVOUS)` or
 * `buffer(0)`, which can deadlock the producer and fails `trySend` while the collector is busy.
 */
internal class CallbackFlowShape(config: Config) : Rule(config, DESCRIPTION) {

    private fun lambdaOf(call: KtCallExpression): KtLambdaExpression? {
        return call.lambdaArguments.firstOrNull()?.getLambdaExpression()
    }

    private fun callsNamed(lambda: KtLambdaExpression, name: String): List<KtCallExpression> {
        return lambda.collectDescendantsOfType<KtCallExpression> { call -> call.calleeExpression?.text == name }
    }

    private fun checkCallbackFlow(call: KtCallExpression) {
        val lambda = lambdaOf(call) ?: return
        if (callsNamed(lambda, AWAIT_CLOSE).isEmpty()) {
            val message = "Unregister the callback in awaitClose { } at the end of this callbackFlow: the builder " +
                "fails without it and the listener leaks."
            report(Finding(Entity.from(call), message))
        }
        callsNamed(lambda, LAUNCH)
            .filter { launch -> lambdaOf(launch)?.let { body -> callsNamed(body, SEND).isNotEmpty() } == true }
            .forEach { launch ->
                val message = "Call trySend(value) from the callback instead of launch { send(value) }: a coroutine " +
                    "per event loses their order."
                report(Finding(Entity.from(launch), message))
            }
    }

    private fun isRendezvous(argument: KtValueArgument): Boolean {
        return argument.getArgumentExpression()?.text in RENDEZVOUS_CAPACITIES
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        when (expression.calleeExpression?.text) {
            CALLBACK_FLOW -> checkCallbackFlow(expression)
            BUFFER -> {
                if (expression.valueArguments.none(::isRendezvous)) return
                val message = "Keep the default buffer, Channel.UNLIMITED or conflate() instead of a rendezvous " +
                    "buffer: the producer can deadlock and trySend fails while the collector is busy."
                report(Finding(Entity.from(expression), message))
            }
        }
    }

    private companion object {
        const val CALLBACK_FLOW = "callbackFlow"
        const val AWAIT_CLOSE = "awaitClose"
        const val LAUNCH = "launch"
        const val SEND = "send"
        const val BUFFER = "buffer"

        @JvmField
        val RENDEZVOUS_CAPACITIES = setOf(
            "0",
            "RENDEZVOUS",
            "Channel.RENDEZVOUS"
        )
        const val DESCRIPTION = "A callbackFlow unregisters in awaitClose, sends with trySend and has a buffer."
    }
}
