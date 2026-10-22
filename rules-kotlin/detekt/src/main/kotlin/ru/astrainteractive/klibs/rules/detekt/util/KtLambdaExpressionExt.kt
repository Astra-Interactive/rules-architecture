package ru.astrainteractive.klibs.rules.detekt.util

import org.jetbrains.kotlin.psi.KtCallExpression
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
