package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtBinaryExpression
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtLambdaExpression
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType
import org.jetbrains.kotlin.psi.psiUtil.getStrictParentOfType

/**
 * Reports a bridge cleanup out of shape: the lambda of `awaitClose` or `invokeOnCancellation` that assigns state
 * declared outside its bridge, which races with the next registration under `flatMapLatest`, or that calls `launch`,
 * which dies silently on the cancelled scope. The bridge is the block that holds the cleanup (the `callbackFlow`,
 * `channelFlow` or `suspendCancellableCoroutine` lambda): a variable declared there belongs to this one registration.
 * The cleanup removes only what its bridge registered.
 */
internal class BridgeCleanup(config: Config) : Rule(config, DESCRIPTION) {

    private fun assignedOutside(cleanup: KtCallExpression, lambda: KtLambdaExpression): List<KtBinaryExpression> {
        val bridge = cleanup.getStrictParentOfType<KtLambdaExpression>() ?: lambda
        val declared = bridge.collectDescendantsOfType<KtProperty>().mapNotNull(KtProperty::getName).toSet()
        return lambda.collectDescendantsOfType<KtBinaryExpression> { expression ->
            expression.operationToken in KtTokens.ALL_ASSIGNMENTS
        }.filter { expression ->
            val name = (expression.left as? KtNameReferenceExpression)?.getReferencedName()
            name == null || name !in declared
        }
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        val name = expression.calleeExpression?.text
        if (name !in CLEANUP_CALLS) return
        val lambda = expression.lambdaArguments.firstOrNull()?.getLambdaExpression() ?: return
        assignedOutside(expression, lambda).forEach { assignment ->
            val assigned = assignment.left?.text
            val message = "Keep the $name cleanup to what this bridge registered: it assigns '$assigned' declared " +
                "outside it, which races with the next registration."
            report(Finding(Entity.from(assignment), message))
        }
        lambda.collectDescendantsOfType<KtCallExpression> { call -> call.calleeExpression?.text == LAUNCH }
            .forEach { launch ->
                val message = "Unregister synchronously in $name: a launch there runs on the cancelled scope and " +
                    "dies silently; a suspending unregister runs on a scope the class owns."
                report(Finding(Entity.from(launch), message))
            }
    }

    private companion object {
        const val LAUNCH = "launch"

        @JvmField
        val CLEANUP_CALLS = setOf("awaitClose", "invokeOnCancellation")
        const val DESCRIPTION = "The cleanup of a bridge removes only what it registered, synchronously."
    }
}
