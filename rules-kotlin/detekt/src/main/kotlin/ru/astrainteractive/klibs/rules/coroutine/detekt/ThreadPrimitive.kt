package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtElement
import ru.astrainteractive.klibs.rules.detekt.util.isTestSource

/**
 * Reports `Thread` in the main sources, constructed (`Thread { … }`) or used through its static members
 * (`Thread.sleep(…)`): application code launches coroutines instead.
 */
internal class ThreadPrimitive(config: Config) : Rule(config, DESCRIPTION) {

    private fun reportThread(element: KtElement) {
        if (element.containingKtFile.isTestSource()) return
        val message = "Launch a coroutine instead of using Thread: application code runs work in coroutines."
        report(Finding(Entity.from(element), message))
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        if (expression.calleeExpression?.text != THREAD) return
        reportThread(expression)
    }

    override fun visitDotQualifiedExpression(expression: KtDotQualifiedExpression) {
        super.visitDotQualifiedExpression(expression)
        if (expression.receiverExpression.text !in THREAD_RECEIVERS) return
        reportThread(expression)
    }

    private companion object {
        const val THREAD = "Thread"

        @JvmField
        val THREAD_RECEIVERS = setOf(THREAD, "java.lang.Thread")
        const val DESCRIPTION = "Application code does not use Thread."
    }
}
