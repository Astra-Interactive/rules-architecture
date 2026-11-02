package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtFunction
import org.jetbrains.kotlin.psi.KtLambdaExpression
import org.jetbrains.kotlin.psi.KtTryExpression
import org.jetbrains.kotlin.psi.KtValueArgument
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType
import org.jetbrains.kotlin.psi.psiUtil.getStrictParentOfType

/**
 * Reports a callback bridge out of the no-callbacks shape: a `callbackFlow` whose block does not end with
 * `awaitClose { … }` (or `try { … awaitClose() } finally { … }`), which throws `IllegalStateException` at run time
 * when the block completes and leaks the listener; a `launch { send }` inside a callback of it, which loses the
 * order and starts a coroutine per event (a `launch` of the producer block itself runs once and is not judged); a
 * `buffer(Channel.RENDEZVOUS)` or `buffer(0)`, which can deadlock the producer.
 * The sharing policy depends on the resource's lifetime and is reviewed manually.
 */
internal class CallbackFlowShape(config: Config) : Rule(config, DESCRIPTION) {

    private fun lambdaOf(call: KtCallExpression): KtLambdaExpression? {
        return call.lambdaArguments.firstOrNull()?.getLambdaExpression()
    }

    private fun callsNamed(lambda: KtLambdaExpression, name: String): List<KtCallExpression> {
        return lambda.collectDescendantsOfType<KtCallExpression> { call -> call.calleeExpression?.text == name }
    }

    private fun isInCallback(launch: KtCallExpression, producer: KtLambdaExpression): Boolean {
        return launch.getStrictParentOfType<KtFunction>() != producer.functionLiteral
    }

    private fun isAwaitClose(statement: KtExpression?): Boolean {
        return (statement as? KtCallExpression)?.calleeExpression?.text == AWAIT_CLOSE
    }

    private fun endsWithAwaitClose(lambda: KtLambdaExpression): Boolean {
        val last = lambda.bodyExpression?.statements?.lastOrNull()
        if (isAwaitClose(last)) return true
        val tryBlock = (last as? KtTryExpression)?.tryBlock ?: return false
        return isAwaitClose(tryBlock.statements.lastOrNull())
    }

    private fun checkCallbackFlow(call: KtCallExpression) {
        val lambda = lambdaOf(call) ?: return
        if (!endsWithAwaitClose(lambda)) {
            val message = "End this callbackFlow block with awaitClose { unregister }: the builder throws " +
                "IllegalStateException at run time when the block completes without it, and the listener leaks."
            report(Finding(Entity.from(call), message))
        }
        callsNamed(lambda, LAUNCH)
            .filter { launch -> isInCallback(launch, lambda) }
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

    private fun isCallbackBridgeBuffer(call: KtCallExpression): Boolean {
        val qualified = call.parent as? KtDotQualifiedExpression ?: return false
        if (qualified.selectorExpression != call) return false
        return generateSequence(qualified.receiverExpression) { receiver ->
            (receiver as? KtDotQualifiedExpression)?.receiverExpression
        }.any { receiver ->
            val producer = when (receiver) {
                is KtCallExpression -> receiver
                is KtDotQualifiedExpression -> receiver.selectorExpression as? KtCallExpression
                else -> null
            }
            producer?.calleeExpression?.text == CALLBACK_FLOW
        }
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        when (expression.calleeExpression?.text) {
            CALLBACK_FLOW -> checkCallbackFlow(expression)
            BUFFER -> {
                if (!isCallbackBridgeBuffer(expression) || expression.valueArguments.none(::isRendezvous)) return
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
        const val DESCRIPTION = "A callbackFlow ends with awaitClose, sends with trySend and has a buffer."
    }
}
