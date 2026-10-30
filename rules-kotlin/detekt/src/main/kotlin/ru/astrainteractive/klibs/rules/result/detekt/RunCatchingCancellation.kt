package ru.astrainteractive.klibs.rules.result.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtIfExpression
import org.jetbrains.kotlin.psi.KtIsExpression
import org.jetbrains.kotlin.psi.KtLambdaExpression
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtQualifiedExpression
import org.jetbrains.kotlin.psi.psiUtil.parents
import ru.astrainteractive.klibs.rules.detekt.util.findCallName
import ru.astrainteractive.klibs.rules.detekt.util.hasThrowBranch

/**
 * Reports a `runCatching` in coroutine code that is not followed in the same call chain by
 * `.propagateCancellationException()` or, in a project without mikro, by the inline
 * `.onFailure { error -> if (error is CancellationException) throw error }`. Coroutine code is the body of a
 * `suspend` function and the lambda of a coroutine builder or a flow operator (`launch`, `async`, `withContext`,
 * `flow`, `collect`, `runTest`, …), lambdas inside them included.
 */
internal class RunCatchingCancellation(config: Config) : Rule(config, DESCRIPTION) {

    private fun isInCoroutine(call: KtCallExpression): Boolean {
        for (parent in call.parents) {
            if (parent is KtLambdaExpression && parent.findCallName() in COROUTINE_CALLS) return true
            if (parent is KtNamedFunction) return parent.hasModifier(KtTokens.SUSPEND_KEYWORD)
            if (parent is KtClassOrObject) return false
        }
        return false
    }

    private fun isInlinePropagation(onFailure: KtCallExpression): Boolean {
        val lambda = onFailure.lambdaArguments.singleOrNull()?.getLambdaExpression() ?: return false
        val check = lambda.bodyExpression?.statements?.singleOrNull() as? KtIfExpression ?: return false
        val condition = check.condition as? KtIsExpression ?: return false
        val typeName = condition.typeReference?.text.orEmpty().substringAfterLast('.')
        return !condition.isNegated && typeName == CANCELLATION_EXCEPTION && check.hasThrowBranch()
    }

    private fun isFollowedByPropagation(call: KtCallExpression): Boolean {
        val callParent = call.parent as? KtQualifiedExpression
        val chainElement: KtExpression = callParent?.takeIf { parent -> parent.selectorExpression == call } ?: call
        val next = chainElement.parent as? KtQualifiedExpression ?: return false
        if (next.receiverExpression != chainElement) return false
        val selector = next.selectorExpression as? KtCallExpression ?: return false
        return when (selector.calleeExpression?.text) {
            PROPAGATE -> true
            ON_FAILURE -> isInlinePropagation(selector)
            else -> false
        }
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        if (expression.calleeExpression?.text != RUN_CATCHING) return
        if (!isInCoroutine(expression) || isFollowedByPropagation(expression)) return
        val message = "runCatching in coroutine code catches CancellationException: call " +
            ".propagateCancellationException() right after it, or without mikro " +
            ".onFailure { error -> if (error is CancellationException) throw error }."
        report(Finding(Entity.from(expression), message))
    }

    private companion object {
        const val RUN_CATCHING = "runCatching"
        const val PROPAGATE = "propagateCancellationException"
        const val ON_FAILURE = "onFailure"
        const val CANCELLATION_EXCEPTION = "CancellationException"

        @JvmField
        val COROUTINE_CALLS = setOf(
            "launch",
            "async",
            "withContext",
            "coroutineScope",
            "supervisorScope",
            "flow",
            "collect",
            "collectLatest",
            "runTest"
        )
        const val DESCRIPTION = "runCatching in coroutine code is followed by .propagateCancellationException()."
    }
}
