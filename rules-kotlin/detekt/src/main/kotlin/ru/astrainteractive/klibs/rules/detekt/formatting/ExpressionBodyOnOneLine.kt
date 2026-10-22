package ru.astrainteractive.klibs.rules.detekt.formatting

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtNamedFunction

/**
 * Reports a function with an expression body (`= …`) that does not stay on one line: a line break after the `=`
 * or inside the body.
 */
internal class ExpressionBodyOnOneLine(config: Config) : Rule(config, DESCRIPTION) {

    private fun isBodyOnOneLine(function: KtNamedFunction): Boolean {
        val equals = function.equalsToken ?: return true
        val body = function.bodyExpression ?: return true
        val bodyEnd = body.textRange.endOffset - function.textRange.startOffset
        val equalsStart = equals.textRange.startOffset - function.textRange.startOffset
        return '\n' !in function.text.substring(equalsStart, bodyEnd)
    }

    override fun visitNamedFunction(function: KtNamedFunction) {
        super.visitNamedFunction(function)
        if (function.hasBlockBody()) return
        if (isBodyOnOneLine(function)) return
        val message = "The expression body of '${function.name}' spans several lines: use a block body with an " +
            "explicit return."
        report(Finding(Entity.from(function), message))
    }

    private companion object {
        const val DESCRIPTION = "An expression body is used only when the whole function fits on one line."
    }
}
