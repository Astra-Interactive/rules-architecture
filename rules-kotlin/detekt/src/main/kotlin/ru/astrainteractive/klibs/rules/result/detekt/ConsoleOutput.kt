package ru.astrainteractive.klibs.rules.result.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtCallableReferenceExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.psiUtil.getQualifiedExpressionForSelector

/**
 * Reports output written to the console, in the main and the test sources alike: a call of `print` or `println`
 * without a receiver or through `kotlin.io`, `print`, `println` and `printf` on `System.out` or `System.err`, and
 * `printStackTrace()` without arguments on any receiver; a function reference to any of them (`::println`) too.
 *
 * The rule reads the source only: an unqualified `print`/`println` is reported whatever it resolves to, a function
 * of the project with that name included. `printStackTrace(writer)` writes to the given writer and passes. Detekt's
 * `exceptions.PrintStackTrace` sees `printStackTrace` only on the parameter of the enclosing `catch`; this rule
 * reports every call site.
 */
internal class ConsoleOutput(config: Config) : Rule(config, DESCRIPTION) {

    private fun isSystemStream(receiver: KtExpression?): Boolean {
        return receiver?.text?.filterNot(Char::isWhitespace) in SYSTEM_STREAMS
    }

    private fun isConsoleOutput(name: String?, receiver: KtExpression?): Boolean {
        return when (name) {
            PRINT_STACK_TRACE -> true
            PRINTF -> isSystemStream(receiver)
            in PRINT_FUNCTIONS -> receiver == null || receiver.text == KOTLIN_IO || isSystemStream(receiver)
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
        const val PRINTF = "printf"
        const val KOTLIN_IO = "kotlin.io"

        @JvmField
        val PRINT_FUNCTIONS = setOf("print", "println")

        @JvmField
        val SYSTEM_STREAMS = setOf(
            "System.out",
            "System.err",
            "java.lang.System.out",
            "java.lang.System.err"
        )
        const val DESCRIPTION = "Every message goes through the logger, never to the console."
    }
}
