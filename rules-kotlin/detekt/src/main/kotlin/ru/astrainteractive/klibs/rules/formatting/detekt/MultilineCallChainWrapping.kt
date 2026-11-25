package ru.astrainteractive.klibs.rules.formatting.detekt

import com.intellij.psi.PsiElement
import com.intellij.psi.util.PsiTreeUtil
import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtBinaryExpression
import org.jetbrains.kotlin.psi.KtBinaryExpressionWithTypeRHS
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtOperationExpression
import org.jetbrains.kotlin.psi.KtQualifiedExpression
import org.jetbrains.kotlin.psi.KtTypeReference

internal class MultilineCallChainWrapping(config: Config) : Rule(config, DESCRIPTION) {

    private fun findReceiver(expression: PsiElement): KtExpression? {
        return when (expression) {
            is KtQualifiedExpression -> expression.receiverExpression

            is KtBinaryExpressionWithTypeRHS -> {
                if (expression.operationReference.getReferencedNameElementType() != KtTokens.AS_SAFE) return null
                expression.left
            }

            is KtBinaryExpression -> {
                if (expression.operationToken != KtTokens.ELVIS) return null
                expression.left
            }

            else -> null
        }
    }

    private fun findStep(expression: PsiElement): Step? {
        val operator = when (expression) {
            is KtQualifiedExpression -> expression.operationTokenNode.psi
            is KtOperationExpression -> expression.operationReference
            else -> return null
        }
        val receiver = findReceiver(expression) ?: return null
        val following = PsiTreeUtil.skipWhitespacesAndCommentsForward(operator)
            .takeIf { operand -> operand is KtExpression || operand is KtTypeReference }
        val isQualified = expression is KtQualifiedExpression
        val isCall = isQualified && following is KtCallExpression
        return Step(
            receiver = receiver,
            operator = operator,
            following = following,
            isProperty = isQualified && !isCall,
            isCall = isCall
        )
    }

    private fun hasLineBreakBetween(left: PsiElement, right: PsiElement): Boolean {
        val source = left.containingFile.viewProvider.contents
        val gap = source.subSequence(left.textRange.endOffset, right.textRange.startOffset)
        return gap.contains('\n') || gap.contains('\r')
    }

    private fun isWrapped(step: Step): Boolean {
        return hasLineBreakBetween(step.receiver, step.following ?: step.operator)
    }

    override fun visitExpression(expression: KtExpression) {
        super.visitExpression(expression)
        val step = findStep(expression) ?: return
        if (findStep(expression.parent)?.receiver == expression) return
        val steps = generateSequence(step) { current -> findStep(current.receiver) }.toList().asReversed()
        var hasCall = steps.first().receiver is KtCallExpression
        if (!hasCall && steps.none(Step::isCall)) return
        if (steps.none(::isWrapped)) return
        for (current in steps.filterNot(Step::isProperty)) {
            if (current.isCall && !hasCall) {
                hasCall = true
            } else if (!hasLineBreakBetween(current.receiver, current.operator)) {
                report(Finding(Entity.from(current.operator), DESCRIPTION))
            }
        }
    }

    private data class Step(
        val receiver: KtExpression,
        val operator: PsiElement,
        val following: PsiElement?,
        val isProperty: Boolean,
        val isCall: Boolean
    )

    private companion object {
        const val DESCRIPTION = "In a wrapped call chain, start each call after the first, each as? and each ?: " +
            "on a new line."
    }
}
