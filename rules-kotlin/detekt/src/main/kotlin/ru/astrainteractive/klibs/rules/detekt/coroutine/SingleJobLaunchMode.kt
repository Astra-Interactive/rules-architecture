package ru.astrainteractive.klibs.rules.detekt.coroutine

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtQualifiedExpression
import org.jetbrains.kotlin.psi.psiUtil.containingClassOrObject
import org.jetbrains.kotlin.psi.psiUtil.getStrictParentOfType
import ru.astrainteractive.klibs.rules.detekt.util.findEnclosingProperty

/**
 * Reports a launch from a user action that is not guarded against a second launch: `scope.launch(…)` on a
 * single-job scope (`.asSingleJobScope()`) without a `SingleJobMode`, and a launch on the plain scope of a view
 * model or component from one of its `on…` handlers.
 */
internal class SingleJobLaunchMode(config: Config) : Rule(config, DESCRIPTION) {

    private fun isSingleJobScope(name: String, call: KtCallExpression): Boolean {
        val initializer = call.findEnclosingProperty(name)?.initializer as? KtDotQualifiedExpression ?: return false
        val selector = initializer.selectorExpression as? KtCallExpression ?: return false
        return selector.calleeExpression?.text == AS_SINGLE_JOB_SCOPE
    }

    private fun hasMode(call: KtCallExpression): Boolean {
        return call.valueArguments.any { argument ->
            argument.getArgumentExpression()?.text.orEmpty().startsWith(SINGLE_JOB_MODE)
        }
    }

    private fun isUserActionHandler(call: KtCallExpression): Boolean {
        val function = call.getStrictParentOfType<KtNamedFunction>() ?: return false
        if (!HANDLER_NAME.matches(function.name.orEmpty())) return false
        val owner = function.containingClassOrObject?.name.orEmpty()
        return STATE_HOLDER_SUFFIXES.any(owner::endsWith)
    }

    private fun findMessage(scopeName: String, call: KtCallExpression): String? {
        if (isSingleJobScope(scopeName, call)) {
            if (hasMode(call)) return null
            return "Pass the SingleJobMode to '$scopeName.launch': skip while running for a purchase, cancel the " +
                "previous for a search; never leave it to the default."
        }
        if (scopeName !in PLAIN_SCOPES || !isUserActionHandler(call)) return null
        return "Guard this launch against a second click: launch on a single-job scope " +
            "('$scopeName.asSingleJobScope()' with a SingleJobMode), or claim the state with getAndUpdate first."
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
        const val SINGLE_JOB_MODE = "SingleJobMode."

        @JvmField
        val HANDLER_NAME = Regex("on[A-Z].*")

        @JvmField
        val STATE_HOLDER_SUFFIXES = listOf("ViewModel", "Component")

        @JvmField
        val PLAIN_SCOPES = setOf(
            "viewModelScope",
            "componentScope",
            "coroutineScope",
            "scope"
        )
        const val DESCRIPTION = "A coroutine launched from a user action is guarded atomically against a second launch."
    }
}
