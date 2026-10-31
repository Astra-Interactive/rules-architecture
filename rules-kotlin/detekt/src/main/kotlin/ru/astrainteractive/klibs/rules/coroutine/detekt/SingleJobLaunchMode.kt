package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtQualifiedExpression
import ru.astrainteractive.klibs.rules.detekt.util.findEnclosingProperty

/**
 * Reports a launch on a directly declared single-job scope without an explicit mode.
 * Atomic state claims and mutex protection of ordinary scopes are reviewed separately.
 */
internal class SingleJobLaunchMode(config: Config) : Rule(config, DESCRIPTION) {

    private fun isSingleJobScope(name: String, call: KtCallExpression): Boolean {
        val initializer = call.findEnclosingProperty(name)?.initializer as? KtDotQualifiedExpression ?: return false
        val selector = initializer.selectorExpression as? KtCallExpression ?: return false
        return selector.calleeExpression?.text == AS_SINGLE_JOB_SCOPE
    }

    private fun hasMode(call: KtCallExpression): Boolean {
        return call.valueArgumentList?.arguments.orEmpty().any { argument ->
            val argumentName = argument.getArgumentName()?.asName?.asString()
            argumentName == null || argumentName == MODE
        }
    }

    private fun findMessage(scopeName: String, call: KtCallExpression): String? {
        if (isSingleJobScope(scopeName, call)) {
            if (hasMode(call)) return null
            return "Pass the SingleJobMode to '$scopeName.launch': skip while running for a purchase, cancel the " +
                "previous for a search; never leave it to the default."
        }
        return null
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        if (expression.calleeExpression?.text != LAUNCH) return
        val qualified = expression.parent as? KtQualifiedExpression ?: return
        if (qualified.selectorExpression != expression) return
        val scope = qualified.receiverExpression as? KtNameReferenceExpression ?: return
        val message = findMessage(scope.getReferencedName(), expression) ?: return
        report(Finding(Entity.from(expression), message))
    }

    private companion object {
        const val LAUNCH = "launch"
        const val AS_SINGLE_JOB_SCOPE = "asSingleJobScope"
        const val MODE = "mode"

        const val DESCRIPTION = "A single-job launch selects its mode explicitly."
    }
}
