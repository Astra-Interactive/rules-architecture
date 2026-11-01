package ru.astrainteractive.klibs.rules.detekt.util

import org.jetbrains.kotlin.analysis.api.analyze
import org.jetbrains.kotlin.analysis.api.resolution.singleFunctionCallOrNull
import org.jetbrains.kotlin.analysis.api.resolution.symbol
import org.jetbrains.kotlin.analysis.api.symbols.KaConstructorSymbol
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtProperty

private fun KtExpression.asCall(): KtCallExpression? {
    return when (this) {
        is KtCallExpression -> this
        is KtDotQualifiedExpression -> selectorExpression as? KtCallExpression
        else -> null
    }
}

private fun KtCallExpression.isConstructor(): Boolean {
    return analyze(this) { resolveToCall()?.singleFunctionCallOrNull()?.symbol is KaConstructorSymbol }
}

/**
 * The constructor this property directly owns, including a single constructor returned by Kotlin's `lazy`.
 * Injected references, aliases, factories and ambiguous expressions provide no construction evidence.
 */
internal fun KtProperty.ownedConstructionOrNull(): KtCallExpression? {
    val initial = initializer?.asCall()
    if (initial?.isConstructor() == true) return initial
    val delegate = delegateExpression?.asCall() ?: return null
    val isLazy = analyze(delegate) {
        val symbol = delegate.resolveToCall()?.singleFunctionCallOrNull()?.symbol
        symbol?.callableId?.asSingleFqName()?.asString() == "kotlin.lazy"
    }
    if (!isLazy) return null
    val lambda = delegate.lambdaArguments.singleOrNull()?.getLambdaExpression() ?: return null
    val result = lambda.bodyExpression?.statements?.singleOrNull()?.asCall() ?: return null
    return result.takeIf(KtCallExpression::isConstructor)
}
