package ru.astrainteractive.klibs.rules.detekt.coroutine

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtQualifiedExpression
import org.jetbrains.kotlin.psi.KtThisExpression
import ru.astrainteractive.klibs.rules.detekt.util.findEnclosingProperty

/**
 * Reports `x.emit(…)` and `x.tryEmit(…)` where `x` is a property of the enclosing class or file declared as a
 * `MutableStateFlow<…>` or initialised with `MutableStateFlow(…)`.
 */
internal class MutableStateFlowEmit(config: Config) : Rule(config, DESCRIPTION) {

    private fun receiverName(receiver: KtExpression): String? {
        if (receiver is KtNameReferenceExpression) return receiver.getReferencedName()
        if (receiver !is KtDotQualifiedExpression || receiver.receiverExpression !is KtThisExpression) return null
        return (receiver.selectorExpression as? KtNameReferenceExpression)?.getReferencedName()
    }

    private fun isMutableStateFlow(property: KtProperty): Boolean {
        val typeText = property.typeReference?.text.orEmpty()
        if (typeText.startsWith(MUTABLE_STATE_FLOW)) return true
        val initializer = property.initializer as? KtCallExpression ?: return false
        return initializer.calleeExpression?.text == MUTABLE_STATE_FLOW
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        val callName = expression.calleeExpression?.text
        if (callName !in EMIT_NAMES) return
        val qualified = expression.parent as? KtQualifiedExpression ?: return
        if (qualified.selectorExpression != expression) return
        val name = receiverName(qualified.receiverExpression) ?: return
        val property = expression.findEnclosingProperty(name) ?: return
        if (!isMutableStateFlow(property)) return
        val message = "'$name' is a MutableStateFlow: write it with '.value =', or with update { } when the new " +
            "value depends on the current one, never with $callName."
        report(Finding(Entity.from(expression), message))
    }

    private companion object {
        const val MUTABLE_STATE_FLOW = "MutableStateFlow"

        @JvmField
        val EMIT_NAMES = setOf("emit", "tryEmit")
        const val DESCRIPTION = "A MutableStateFlow is written with .value or update, never with emit or tryEmit."
    }
}
