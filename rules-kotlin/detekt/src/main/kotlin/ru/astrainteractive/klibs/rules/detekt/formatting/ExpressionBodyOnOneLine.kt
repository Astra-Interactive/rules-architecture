package ru.astrainteractive.klibs.rules.detekt.formatting

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.lexer.KtModifierKeywordToken
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.psiUtil.allChildren

/**
 * Reports a function with an expression body (`= …`) whose declaration, from its first modifier (or `fun`) to
 * the end of the body, does not fit on one line: a wrapped signature, a line break after the `=` or inside the
 * body. Annotations and KDoc above the declaration do not count.
 */
internal class ExpressionBodyOnOneLine(config: Config) : Rule(config, DESCRIPTION) {

    private fun declarationStart(function: KtNamedFunction): Int {
        val firstModifier = function.modifierList?.allChildren
            ?.firstOrNull { child -> child.node.elementType is KtModifierKeywordToken }
        val start = firstModifier ?: function.funKeyword ?: function
        return start.textRange.startOffset
    }

    private fun isOnOneLine(function: KtNamedFunction): Boolean {
        val body = function.bodyExpression ?: return true
        val fileText = function.containingFile.text
        return '\n' !in fileText.substring(declarationStart(function), body.textRange.endOffset)
    }

    override fun visitNamedFunction(function: KtNamedFunction) {
        super.visitNamedFunction(function)
        if (function.hasBlockBody() || function.equalsToken == null) return
        if (isOnOneLine(function)) return
        val message = "The expression-body function '${function.name}' spans several lines: use a block body " +
            "with an explicit return; an expression body is one line, signature and body."
        report(Finding(Entity.from(function), message))
    }

    private companion object {
        const val DESCRIPTION = "An expression body is used only when the signature and the body are one line."
    }
}
