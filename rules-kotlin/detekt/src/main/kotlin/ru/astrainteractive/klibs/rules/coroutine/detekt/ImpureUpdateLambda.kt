package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtArrayAccessExpression
import org.jetbrains.kotlin.psi.KtBinaryExpression
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtDestructuringDeclarationEntry
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtLambdaExpression
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtNamedDeclaration
import org.jetbrains.kotlin.psi.KtParameter
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtQualifiedExpression
import org.jetbrains.kotlin.psi.KtThisExpression
import org.jetbrains.kotlin.psi.KtUnaryExpression
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports an assignment inside the lambda of `update`, `getAndUpdate` or `updateAndGet` to a name declared outside
 * that lambda: the lambda is a compare-and-set loop that may run twice, so it stays pure.
 */
internal class ImpureUpdateLambda(config: Config) : Rule(config, DESCRIPTION) {

    private fun assignedName(target: KtExpression?): String? {
        return when (target) {
            is KtNameReferenceExpression -> target.getReferencedName()
            is KtArrayAccessExpression -> assignedName(target.arrayExpression)
            is KtQualifiedExpression -> {
                val receiver = target.receiverExpression
                if (receiver is KtThisExpression) assignedName(target.selectorExpression) else assignedName(receiver)
            }
            else -> null
        }
    }

    private fun assignedNames(lambda: KtLambdaExpression): List<Assignment> {
        val assignments = lambda.collectDescendantsOfType<KtBinaryExpression> { expression ->
            expression.operationToken in KtTokens.ALL_ASSIGNMENTS
        }.map { expression -> Assignment(expression, assignedName(expression.left)) }
        val increments = lambda.collectDescendantsOfType<KtUnaryExpression> { expression ->
            expression.operationToken == KtTokens.PLUSPLUS || expression.operationToken == KtTokens.MINUSMINUS
        }.map { expression -> Assignment(expression, assignedName(expression.baseExpression)) }
        return assignments + increments
    }

    private fun declaredNames(lambda: KtLambdaExpression): Set<String?> {
        val parameters = lambda.collectDescendantsOfType<KtParameter>()
        val properties = lambda.collectDescendantsOfType<KtProperty>()
        val destructured = lambda.collectDescendantsOfType<KtDestructuringDeclarationEntry>()
        val declarations = parameters + properties + destructured
        return declarations.map(KtNamedDeclaration::getName).toSet()
    }

    private fun lambdasOf(call: KtCallExpression): List<KtLambdaExpression> {
        return call.valueArguments.mapNotNull { argument -> argument.getArgumentExpression() as? KtLambdaExpression }
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        val callName = expression.calleeExpression?.text
        if (callName !in UPDATE_NAMES) return
        lambdasOf(expression).forEach { lambda ->
            val declared = declaredNames(lambda)
            assignedNames(lambda)
                .filter { assignment -> assignment.name != null && assignment.name !in declared }
                .forEach { assignment ->
                    val message = "The $callName lambda assigns '${assignment.name}', declared outside it: the " +
                        "lambda may run twice, so keep it pure and use getAndUpdate with a pure transition instead."
                    report(Finding(Entity.from(assignment.expression), message))
                }
        }
    }

    private class Assignment(val expression: KtExpression, val name: String?)

    private companion object {
        @JvmField
        val UPDATE_NAMES = setOf(
            "update",
            "getAndUpdate",
            "updateAndGet"
        )
        const val DESCRIPTION = "The lambda of update, getAndUpdate and updateAndGet assigns nothing outside itself."
    }
}
