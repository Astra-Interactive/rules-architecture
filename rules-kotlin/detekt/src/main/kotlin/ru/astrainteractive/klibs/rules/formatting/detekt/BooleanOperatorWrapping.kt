package ru.astrainteractive.klibs.rules.formatting.detekt

import com.intellij.psi.PsiElement
import com.intellij.psi.PsiWhiteSpace
import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtBinaryExpression

/**
 * Reports a line break on either side of `&&` or `||`: a condition that does not fit on one line is split, not
 * wrapped at its operators.
 */
internal class BooleanOperatorWrapping(config: Config) : Rule(config, DESCRIPTION) {

    private fun isLineBreak(element: PsiElement?): Boolean = element is PsiWhiteSpace && '\n' in element.text

    override fun visitBinaryExpression(expression: KtBinaryExpression) {
        super.visitBinaryExpression(expression)
        val operation = expression.operationReference
        if (expression.operationToken != KtTokens.ANDAND && expression.operationToken != KtTokens.OROR) return
        if (!isLineBreak(operation.prevSibling) && !isLineBreak(operation.nextSibling)) return
        val message = "The condition is wrapped at '${operation.text}': split it into guard clauses or named vals, " +
            "or chain the named values with .and()/.or(), one step per line."
        report(Finding(Entity.from(operation), message))
    }

    private companion object {
        const val DESCRIPTION = "A condition is never wrapped at && or ||."
    }
}
