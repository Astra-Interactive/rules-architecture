package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtLambdaExpression
import org.jetbrains.kotlin.psi.KtTryExpression
import org.jetbrains.kotlin.psi.KtValueArgument
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports a callback bridge out of the no-callbacks shape: a `callbackFlow` whose block does not end with
 * `awaitClose { … }` (or `try { … awaitClose() } finally { … }`), which throws `IllegalStateException` at run time
 * when the block completes and leaks the listener; a `launch { send }` inside it, which loses the order and starts a
 * coroutine per event; a `buffer(Channel.RENDEZVOUS)` or `buffer(0)`, which can deadlock the producer; and a bridge
 * shared with `shareIn`/`stateIn` started otherwise than `SharingStarted.WhileSubscribed()`.
 */
internal class CallbackFlowShape(config: Config) : Rule(config, DESCRIPTION) {

    private fun lambdaOf(call: KtCallExpression): KtLambdaExpression? {
        return call.lambdaArguments.firstOrNull()?.getLambdaExpression()
    }

    private fun callsNamed(lambda: KtLambdaExpression, name: String): List<KtCallExpression> {
        return lambda.collectDescendantsOfType<KtCallExpression> { call -> call.calleeExpression?.text == name }
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

    private fun findSharingStart(call: KtCallExpression): KtExpression? {
        val named = call.valueArguments.firstOrNull { argument ->
            argument.getArgumentName()?.asName?.asString() == STARTED
        }
        val positional = call.valueArguments.getOrNull(STARTED_POSITION)?.takeIf { argument ->
            argument.getArgumentName() == null
        }
        return (named ?: positional)?.getArgumentExpression()
    }

    private fun checkSharing(callbackFlow: KtCallExpression) {
        var element: KtExpression = callbackFlow
        while (true) {
            val chain = element.parent as? KtDotQualifiedExpression ?: return
            if (chain.receiverExpression != element) return
            val selector = chain.selectorExpression as? KtCallExpression
            val started = selector?.takeIf { call -> call.calleeExpression?.text in SHARING_CALLS }
                ?.let(::findSharingStart)
            if (started != null && WHILE_SUBSCRIBED !in started.text) {
                val message = "Share this bridge with SharingStarted.WhileSubscribed(): it registers the callback " +
                    "while someone collects and unregisters after the last collector leaves."
                report(Finding(Entity.from(started), message))
            }
            element = chain
        }
    }

    private fun checkCallbackFlow(call: KtCallExpression) {
        val lambda = lambdaOf(call) ?: return
        if (!endsWithAwaitClose(lambda)) {
            val message = "End this callbackFlow block with awaitClose { unregister }: the builder throws " +
                "IllegalStateException at run time when the block completes without it, and the listener leaks."
            report(Finding(Entity.from(call), message))
        }
        callsNamed(lambda, LAUNCH)
            .filter { launch -> lambdaOf(launch)?.let { body -> callsNamed(body, SEND).isNotEmpty() } == true }
            .forEach { launch ->
                val message = "Call trySend(value) from the callback instead of launch { send(value) }: a coroutine " +
                    "per event loses their order."
                report(Finding(Entity.from(launch), message))
            }
        checkSharing(call)
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
        const val STARTED = "started"
        const val STARTED_POSITION = 1
        const val WHILE_SUBSCRIBED = "WhileSubscribed"

        @JvmField
        val SHARING_CALLS = setOf("shareIn", "stateIn")

        @JvmField
        val RENDEZVOUS_CAPACITIES = setOf(
            "0",
            "RENDEZVOUS",
            "Channel.RENDEZVOUS"
        )
        const val DESCRIPTION = "A callbackFlow ends with awaitClose, sends with trySend, has a buffer and is " +
            "shared while subscribed."
    }
}
