package ru.astrainteractive.klibs.rules.result.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtCallableReferenceExpression
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.psiUtil.getQualifiedExpressionForReceiver
import org.jetbrains.kotlin.psi.psiUtil.getQualifiedExpressionForSelector

/**
 * Reports output written to the console, in the main and the test sources alike: a call of `print` or `println`
 * without a receiver or through `kotlin.io`, `printStackTrace()` without arguments on any receiver, a function
 * reference to any of them (`::println`), and every use of `System.out` or `System.err`: a call on it whatever the
 * method (`flush()` included), a reference through it (`System.err::println`), the stream passed or stored
 * (`PrintWriter(System.out)`).
 *
 * The rule reads the source only: an unqualified `print`/`println` is reported whatever it resolves to, a function
 * of the project with that name included. `printStackTrace(writer)` writes to the given writer and passes. Detekt's
 * `exceptions.PrintStackTrace` sees `printStackTrace` only on the parameter of the enclosing `catch`; this rule
 * reports every call site.
 */
internal class ConsoleOutput(config: Config) : Rule(config, DESCRIPTION) {

    private fun isSystemStream(expression: KtDotQualifiedExpression): Boolean {
        if (expression.selectorExpression?.text !in STREAMS) return false
        return expression.receiverExpression.text.filterNot(Char::isWhitespace) in SYSTEM_RECEIVERS
    }

    private fun isConsoleOutput(name: String?, receiver: KtExpression?): Boolean {
        return when (name) {
            PRINT_STACK_TRACE -> true
            in PRINT_FUNCTIONS -> receiver == null || receiver.text == KOTLIN_IO
            else -> false
        }
    }

    private fun reportConsoleOutput(name: String?, element: KtExpression) {
        val message = if (name == PRINT_STACK_TRACE) {
            "Pass the throwable to the logger instead of calling printStackTrace(): logger.warn(\"…\", error) keeps " +
                "the stack trace."
        } else {
            "Write the message through the logger instead of $name: the console is not a log. Only a command-line " +
                "entry point whose product is the console output suppresses ConsoleOutput, with the reason next to it."
        }
        report(Finding(Entity.from(element), message))
    }

    override fun visitDotQualifiedExpression(expression: KtDotQualifiedExpression) {
        super.visitDotQualifiedExpression(expression)
        if (!isSystemStream(expression)) return
        val use = expression.parent as? KtCallableReferenceExpression
            ?: expression.getQualifiedExpressionForReceiver()
            ?: expression
        reportConsoleOutput(expression.text.filterNot(Char::isWhitespace), use)
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        val name = expression.calleeExpression?.text
        if (name == PRINT_STACK_TRACE && expression.valueArguments.isNotEmpty()) return
        val qualified = expression.getQualifiedExpressionForSelector()
        if (!isConsoleOutput(name, qualified?.receiverExpression)) return
        reportConsoleOutput(name, qualified ?: expression)
    }

    override fun visitCallableReferenceExpression(expression: KtCallableReferenceExpression) {
        super.visitCallableReferenceExpression(expression)
        val name = expression.callableReference.getReferencedName()
        if (!isConsoleOutput(name, expression.receiverExpression)) return
        reportConsoleOutput(name, expression)
    }

    private companion object {
        const val PRINT_STACK_TRACE = "printStackTrace"
        const val KOTLIN_IO = "kotlin.io"

        @JvmField
        val PRINT_FUNCTIONS = setOf("print", "println")

        @JvmField
        val STREAMS = setOf("out", "err")

        @JvmField
        val SYSTEM_RECEIVERS = setOf("System", "java.lang.System")
        const val DESCRIPTION = "Every message goes through the logger, never to the console."
    }
}
