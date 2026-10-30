package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtLambdaExpression
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports a `channelFlow` out of its use: one that registers a listener or a callback, which stays registered after
 * the block completes, so that the next `send` hits a closed channel and throws `ClosedSendChannelException` (unless
 * the block ends with `awaitClose { unregister }`, the shape of a `callbackFlow`), and one that launches no coroutine
 * of its own, which is a plain `flow { }`.
 */
internal class ChannelFlowShape(config: Config) : Rule(config, DESCRIPTION) {

    private fun callNames(lambda: KtLambdaExpression): List<String> {
        return lambda.collectDescendantsOfType<KtCallExpression>().mapNotNull { call -> call.calleeExpression?.text }
    }

    private fun endsWithAwaitClose(lambda: KtLambdaExpression): Boolean {
        val last = lambda.bodyExpression?.statements?.lastOrNull() as? KtCallExpression
        return last?.calleeExpression?.text == AWAIT_CLOSE
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        if (expression.calleeExpression?.text != CHANNEL_FLOW) return
        val lambda = expression.lambdaArguments.firstOrNull()?.getLambdaExpression() ?: return
        val names = callNames(lambda)
        val registration = names.firstOrNull(REGISTRATION::matches)
        if (registration != null && !endsWithAwaitClose(lambda)) {
            val message = "Bridge '$registration' with callbackFlow ending in awaitClose { unregister }: channelFlow " +
                "completes with its block, the listener stays registered and its next send throws " +
                "ClosedSendChannelException."
            report(Finding(Entity.from(expression), message))
        }
        if (names.none { name -> name in COROUTINE_BUILDERS }) {
            val message = "Write this producer as flow { }: channelFlow is for a producer that launches " +
                "coroutines of its own."
            report(Finding(Entity.from(expression), message))
        }
    }

    private companion object {
        const val CHANNEL_FLOW = "channelFlow"
        const val AWAIT_CLOSE = "awaitClose"

        @JvmField
        val REGISTRATION = Regex("""(add|register)\w*(Listener|Callback|Observer|Receiver)|subscribe""")

        @JvmField
        val COROUTINE_BUILDERS = setOf("launch", "async")
        const val DESCRIPTION = "A channelFlow launches coroutines of its own and registers no callback."
    }
}
