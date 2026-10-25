package ru.astrainteractive.klibs.rules.lambda.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtFunctionLiteral
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtQualifiedExpression
import org.jetbrains.kotlin.psi.KtSimpleNameExpression
import org.jetbrains.kotlin.psi.KtValueArgumentName
import org.jetbrains.kotlin.psi.psiUtil.parents

/**
 * Reports every use of the implicit lambda parameter `it`.
 *
 * Works on syntax only: an `it` inside a lambda that declares no parameters is taken to be the implicit one.
 */
internal class ImplicitItParameter(config: Config) : Rule(config, DESCRIPTION) {

    private fun isMemberAccess(expression: KtNameReferenceExpression): Boolean {
        val qualified = expression.parent as? KtQualifiedExpression ?: return false
        return qualified.selectorExpression == expression
    }

    private fun isInsideImplicitLambda(expression: KtNameReferenceExpression): Boolean {
        return expression.parents.any { parent ->
            parent is KtFunctionLiteral && !parent.hasParameterSpecification()
        }
    }

    private fun isImplicitIt(expression: KtNameReferenceExpression): Boolean {
        if (expression.getReferencedName() != IMPLICIT_NAME) return false
        if (expression.parent is KtValueArgumentName) return false
        if (isMemberAccess(expression)) return false
        return isInsideImplicitLambda(expression)
    }

    override fun visitSimpleNameExpression(expression: KtSimpleNameExpression) {
        super.visitSimpleNameExpression(expression)
        if (expression !is KtNameReferenceExpression) return
        if (!isImplicitIt(expression)) return
        report(Finding(Entity.from(expression), MESSAGE))
    }

    private companion object {
        const val IMPLICIT_NAME = "it"
        const val DESCRIPTION = "Lambda parameters are named explicitly; the implicit `it` is not used."
        const val MESSAGE = "Name the lambda parameter instead of using the implicit `it`."
    }
}
