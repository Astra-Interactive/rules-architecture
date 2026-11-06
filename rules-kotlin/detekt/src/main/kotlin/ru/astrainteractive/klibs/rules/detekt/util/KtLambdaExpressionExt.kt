package ru.astrainteractive.klibs.rules.detekt.util

import org.jetbrains.kotlin.analysis.api.analyze
import org.jetbrains.kotlin.analysis.api.resolution.singleFunctionCallOrNull
import org.jetbrains.kotlin.analysis.api.resolution.symbol
import org.jetbrains.kotlin.analysis.api.symbols.receiverType
import org.jetbrains.kotlin.analysis.api.types.KaFunctionType
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtLambdaExpression
import org.jetbrains.kotlin.psi.KtValueArgument
import org.jetbrains.kotlin.psi.KtValueArgumentList

/**
 * The name of the call this lambda is an argument of, in parentheses or trailing; `null` for a lambda that is
 * not an argument.
 */
internal fun KtLambdaExpression.findCallName(): String? {
    val argument = parent as? KtValueArgument ?: return null
    val call = argument.parent as? KtCallExpression ?: (argument.parent as? KtValueArgumentList)?.parent
    return (call as? KtCallExpression)?.calleeExpression?.text
}

/**
 * Whether the lambda's resolved type is a `suspend` function type, as a lambda passed to `Flow.map` is.
 */
internal fun KtLambdaExpression.isSuspend(): Boolean {
    return analyze(this) { (expressionType as? KaFunctionType)?.isSuspend == true }
}

/**
 * The function [reference] over [call] as it fits the function type this lambda is passed as: unchanged for a plain
 * function type with one parameter per lambda parameter, named through the receiver type (`<Type>::FaqItem`) for an
 * extension function type whose receiver the callee extends, and `null` when no reference fits: a callback that drops
 * its `it`, a receiver the callee does not extend, an argument of a call that does not resolve.
 */
internal fun KtLambdaExpression.findFittingReference(reference: String, call: KtCallExpression?): String? {
    val lambda = this
    return analyze(lambda) {
        val functionType = lambda.expectedType
        if (functionType !is KaFunctionType) {
            return@analyze reference.takeIf { functionType != null || lambda.parent !is KtValueArgument }
        }
        if (functionType.parameterTypes.size != lambda.valueParameters.size) return@analyze null
        val receiver = functionType.receiverType ?: return@analyze reference
        if (call == null || call.parent is KtDotQualifiedExpression) return@analyze null
        val calleeReceiver = call.resolveToCall()?.singleFunctionCallOrNull()?.symbol?.receiverType
            ?: return@analyze null
        if (!receiver.isSubtypeOf(calleeReceiver)) return@analyze null
        "<Type>$reference"
    }
}
