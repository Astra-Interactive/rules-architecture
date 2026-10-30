package ru.astrainteractive.klibs.rules.formatting.detekt

import com.intellij.psi.PsiComment
import com.intellij.psi.PsiElement
import com.intellij.psi.PsiWhiteSpace
import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtElement
import org.jetbrains.kotlin.psi.KtSuperTypeCallEntry
import org.jetbrains.kotlin.psi.KtValueArgument
import org.jetbrains.kotlin.psi.KtValueArgumentList

/**
 * Reports a call or a constructor call with three or more arguments where not every argument in its parentheses
 * starts on its own line. A trailing lambda counts as an argument and follows the closing parenthesis:
 * `remember(icon, label) { … }` has three.
 */
internal class ArgumentPerLine(config: Config) : Rule(config, DESCRIPTION) {

    private fun startsOnOwnLine(argument: KtValueArgument): Boolean {
        val previous = generateSequence(argument.prevSibling, PsiElement::getPrevSibling)
            .firstOrNull { sibling -> sibling !is PsiComment }
        return previous is PsiWhiteSpace && '\n' in previous.text
    }

    private fun check(
        call: KtElement,
        name: String?,
        argumentList: KtValueArgumentList?,
        trailingLambdaCount: Int
    ) {
        val arguments = argumentList?.arguments.orEmpty()
        val argumentCount = arguments.size.plus(trailingLambdaCount)
        if (argumentCount < MIN_ARGUMENTS) return
        if (arguments.all(::startsOnOwnLine)) return
        val message = "'$name' takes $argumentCount arguments: put each argument on its own line."
        report(Finding(Entity.from(call), message))
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        check(
            expression,
            expression.calleeExpression?.text,
            expression.valueArgumentList,
            expression.lambdaArguments.size
        )
    }

    override fun visitSuperTypeCallEntry(call: KtSuperTypeCallEntry) {
        super.visitSuperTypeCallEntry(call)
        check(
            call,
            call.calleeExpression.text,
            call.valueArgumentList,
            0
        )
    }

    private companion object {
        const val MIN_ARGUMENTS = 3
        const val DESCRIPTION = "A call with three or more arguments puts each argument on its own line."
    }
}
