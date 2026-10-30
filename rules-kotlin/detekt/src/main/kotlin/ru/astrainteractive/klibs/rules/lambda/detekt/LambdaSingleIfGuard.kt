package ru.astrainteractive.klibs.rules.lambda.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtBlockExpression
import org.jetbrains.kotlin.psi.KtIfExpression
import org.jetbrains.kotlin.psi.KtLambdaExpression
import ru.astrainteractive.klibs.rules.detekt.util.findCallName

/**
 * Reports a lambda whose whole body is one `if` without `else` around two or more statements: the condition is
 * inverted into a guard with a labelled return, and the body follows it. An `if` around one statement stays as it
 * is, since the guard would only add lines; the inline `CancellationException` check of the error-handling rule is
 * such an `if`.
 */
internal class LambdaSingleIfGuard(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitLambdaExpression(lambdaExpression: KtLambdaExpression) {
        super.visitLambdaExpression(lambdaExpression)
        val statement = lambdaExpression.bodyExpression?.statements?.singleOrNull() as? KtIfExpression ?: return
        if (statement.`else` != null) return
        val statementCount = (statement.then as? KtBlockExpression)?.statements?.size ?: 1
        if (statementCount < MIN_GUARDED_STATEMENTS) return
        val label = lambdaExpression.findCallName()?.let { name -> "@$name" }.orEmpty()
        val message = "Invert the condition into a guard, 'if (!condition) return$label', and put the body after " +
            "it: a lambda whose whole body is one if is a guard."
        report(Finding(Entity.from(statement), message))
    }

    private companion object {
        const val MIN_GUARDED_STATEMENTS = 2
        const val DESCRIPTION = "A lambda whose whole body is one if around several statements is a guard."
    }
}
