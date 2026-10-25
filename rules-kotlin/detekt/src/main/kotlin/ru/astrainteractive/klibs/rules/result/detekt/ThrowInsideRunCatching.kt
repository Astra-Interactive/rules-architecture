package ru.astrainteractive.klibs.rules.result.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtLambdaExpression
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtThrowExpression
import org.jetbrains.kotlin.psi.psiUtil.parents
import ru.astrainteractive.klibs.rules.detekt.util.findCallName

/**
 * Reports a `throw` inside the lambda of `runCatching`, `mapCatching` or `recoverCatching`: an error is re-labelled
 * with `fold`, never by throwing inside the catching block.
 */
internal class ThrowInsideRunCatching(config: Config) : Rule(config, DESCRIPTION) {

    private fun findCatchingCall(expression: KtThrowExpression): String? {
        return expression.parents
            .takeWhile { parent -> parent !is KtNamedFunction && parent !is KtClassOrObject }
            .filterIsInstance<KtLambdaExpression>()
            .mapNotNull(KtLambdaExpression::findCallName)
            .firstOrNull { name -> name in CATCHING_NAMES }
    }

    override fun visitThrowExpression(expression: KtThrowExpression) {
        super.visitThrowExpression(expression)
        val callName = findCatchingCall(expression) ?: return
        val message = "Do not throw inside $callName: return the value, and turn the failure into a domain error " +
            "with fold after it."
        report(Finding(Entity.from(expression), message))
    }

    private companion object {
        @JvmField
        val CATCHING_NAMES = setOf(
            "runCatching",
            "mapCatching",
            "recoverCatching"
        )
        const val DESCRIPTION = "Nothing is thrown inside runCatching, mapCatching or recoverCatching."
    }
}
