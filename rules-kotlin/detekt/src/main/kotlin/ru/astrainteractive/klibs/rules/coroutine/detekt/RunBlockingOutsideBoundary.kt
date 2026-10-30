package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.psiUtil.parents
import ru.astrainteractive.klibs.rules.detekt.util.isInTechnologyPackage
import ru.astrainteractive.klibs.rules.detekt.util.isTestSource

/**
 * Reports `runBlocking` in the main sources outside a top-level `main` function and outside a technology package,
 * the one place where a platform's blocking callback is bridged. Test sources are left to `RunBlockingInTest`.
 */
internal class RunBlockingOutsideBoundary(config: Config) : Rule(config, DESCRIPTION) {

    private fun isInMainFunction(call: KtCallExpression): Boolean {
        return call.parents
            .filterIsInstance<KtNamedFunction>()
            .any { function -> function.isTopLevel && function.name == MAIN_FUNCTION }
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        if (expression.calleeExpression?.text != RUN_BLOCKING) return
        val file = expression.containingKtFile
        if (file.isTestSource() || file.isInTechnologyPackage()) return
        if (isInMainFunction(expression)) return
        val message = "Make the caller suspend instead of calling runBlocking: it runs only in main and, inside a " +
            "technology package, where the platform demands a blocking callback."
        report(Finding(Entity.from(expression), message))
    }

    private companion object {
        const val RUN_BLOCKING = "runBlocking"
        const val MAIN_FUNCTION = "main"
        const val DESCRIPTION = "runBlocking appears only in main and in a technology package."
    }
}
