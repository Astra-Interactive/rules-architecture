package ru.astrainteractive.klibs.rules.formatting.detekt

import com.intellij.psi.tree.IElementType
import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtBinaryExpression
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtParenthesizedExpression
import org.jetbrains.kotlin.psi.KtQualifiedExpression
import org.jetbrains.kotlin.psi.KtStringTemplateExpression

/**
 * Reports arithmetic in parentheses before a trailing call or property (`(a * b).sp`): one operator is written as
 * its named function (`a.times(b).sp`), two or more are named in a `val` first. A chain of two or more of `plus`,
 * `minus`, `times`, `div` and `rem` calls is reported too. Only arithmetic at the top of the parentheses counts,
 * and string concatenation is left alone.
 */
internal class OperatorBeforeTrailingCall(config: Config) : Rule(config, DESCRIPTION) {

    private fun arithmeticOperations(expression: KtExpression?): List<KtBinaryExpression> {
        val binary = expression as? KtBinaryExpression ?: return emptyList()
        if (binary.operationToken !in OPERATOR_NAMES.keys) return emptyList()
        return listOf(binary) + arithmeticOperations(binary.left) + arithmeticOperations(binary.right)
    }

    private fun isStringConcatenation(binary: KtBinaryExpression): Boolean {
        return binary.left is KtStringTemplateExpression || binary.right is KtStringTemplateExpression
    }

    private fun arithmeticCallName(expression: KtExpression?): String? {
        val qualified = expression as? KtQualifiedExpression ?: return null
        val call = qualified.selectorExpression as? KtCallExpression ?: return null
        return call.calleeExpression?.text?.takeIf { name -> name in OPERATOR_NAMES.values }
    }

    private fun checkParenthesizedReceiver(expression: KtQualifiedExpression) {
        val receiver = expression.receiverExpression as? KtParenthesizedExpression ?: return
        val inner = receiver.expression ?: return
        val operations = arithmeticOperations(inner)
        if (operations.isEmpty() || operations.any(::isStringConcatenation)) return
        val message = if (operations.size == 1) {
            val binary = operations.single()
            val name = OPERATOR_NAMES.getValue(binary.operationToken)
            "Write '${binary.left?.text}.$name(${binary.right?.text})' instead of '${receiver.text}' before the " +
                "trailing call, so the chain reads left to right."
        } else {
            "Name the value of '${receiver.text}' in a val, then make the call on it."
        }
        report(Finding(Entity.from(receiver), message))
    }

    private fun checkArithmeticChain(expression: KtQualifiedExpression) {
        if (arithmeticCallName(expression) == null) return
        if (arithmeticCallName(expression.receiverExpression) == null) return
        val parent = expression.parent as? KtQualifiedExpression
        if (parent?.receiverExpression == expression && arithmeticCallName(parent) != null) return
        val message = "Name the value in a val instead of chaining '${expression.text}': two or more operators " +
            "before a call are named first."
        report(Finding(Entity.from(expression), message))
    }

    override fun visitQualifiedExpression(expression: KtQualifiedExpression) {
        super.visitQualifiedExpression(expression)
        checkParenthesizedReceiver(expression)
        checkArithmeticChain(expression)
    }

    private companion object {
        @JvmField
        val OPERATOR_NAMES: Map<IElementType, String> = mapOf(
            KtTokens.PLUS to "plus",
            KtTokens.MINUS to "minus",
            KtTokens.MUL to "times",
            KtTokens.DIV to "div",
            KtTokens.PERC to "rem"
        )
        const val DESCRIPTION = "Arithmetic before a trailing call is a named function, or a val when it grows."
    }
}
