package ru.astrainteractive.klibs.rules.time.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtValueArgument

/**
 * Reports `.inWholeMilliseconds`, `.inWholeSeconds` or `.inWholeNanoseconds` (with or without `.toInt()` or
 * `.toLong()`) passed to `delay`, `withTimeout`, `withTimeoutOrNull`, `debounce`, `sample` or `timeout`: those
 * have a `Duration` overload, so the `Duration` is passed as it is.
 */
internal class DurationOverloadCall(config: Config) : Rule(config, DESCRIPTION) {

    private fun isWholeUnit(expression: KtExpression?): Boolean {
        val qualified = expression as? KtDotQualifiedExpression ?: return false
        val selector = qualified.selectorExpression
        if (selector is KtCallExpression && selector.calleeExpression?.text in CONVERSIONS) {
            return isWholeUnit(qualified.receiverExpression)
        }
        return selector?.text in WHOLE_UNITS
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        val name = expression.calleeExpression?.text
        if (name !in DURATION_CALLS) return
        expression.valueArguments
            .mapNotNull(KtValueArgument::getArgumentExpression)
            .filter(::isWholeUnit)
            .forEach { argument ->
                val message = "Pass the Duration to $name instead of '${argument.text}': $name has a Duration " +
                    "overload."
                report(Finding(Entity.from(argument), message))
            }
    }

    private companion object {
        @JvmField
        val DURATION_CALLS = setOf(
            "delay",
            "withTimeout",
            "withTimeoutOrNull",
            "debounce",
            "sample",
            "timeout"
        )

        @JvmField
        val WHOLE_UNITS = setOf(
            "inWholeMilliseconds",
            "inWholeSeconds",
            "inWholeNanoseconds"
        )

        @JvmField
        val CONVERSIONS = setOf("toInt", "toLong")
        const val DESCRIPTION = "A call with a Duration overload takes the Duration directly."
    }
}
