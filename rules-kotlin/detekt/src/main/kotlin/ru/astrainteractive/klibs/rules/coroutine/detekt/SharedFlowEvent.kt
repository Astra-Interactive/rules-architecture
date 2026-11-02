package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtQualifiedExpression
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports a `MutableSharedFlow()` without explicit `extraBufferCapacity` fed through `tryEmit`.
 * Whether a stream is a broadcast or a one-off UI event is a review decision.
 */
internal class SharedFlowEvent(config: Config) : Rule(config, DESCRIPTION) {

    private fun sharedFlowConstructor(property: KtProperty): KtCallExpression? {
        val initializer = property.initializer as? KtCallExpression ?: return null
        return initializer.takeIf { call -> call.calleeExpression?.text == MUTABLE_SHARED_FLOW }
    }

    private fun hasBuffer(constructor: KtCallExpression): Boolean {
        val arguments = constructor.valueArguments
        val names = arguments.mapNotNull { argument -> argument.getArgumentName()?.asName?.asString() }
        if (EXTRA_BUFFER_CAPACITY in names) return true
        return names.isEmpty() && arguments.size >= BUFFER_ARGUMENT_POSITION
    }

    private fun isTryEmitOn(call: KtCallExpression, name: String): Boolean {
        if (call.calleeExpression?.text != TRY_EMIT) return false
        val qualified = call.parent as? KtQualifiedExpression ?: return false
        return qualified.selectorExpression == call && qualified.receiverExpression.text in setOf(name, "this.$name")
    }

    override fun visitProperty(property: KtProperty) {
        super.visitProperty(property)
        if (property.isLocal) return
        val constructor = sharedFlowConstructor(property) ?: return
        val name = property.name ?: return
        if (hasBuffer(constructor)) return
        val emits = property.containingKtFile.collectDescendantsOfType<KtCallExpression> { call ->
            isTryEmitOn(call, name)
        }
        if (emits.isEmpty()) return
        val message = "Give '$name' an extraBufferCapacity: tryEmit on a MutableSharedFlow without a buffer drops " +
            "every value."
        report(Finding(Entity.atName(property), message))
    }

    private companion object {
        const val MUTABLE_SHARED_FLOW = "MutableSharedFlow"
        const val EXTRA_BUFFER_CAPACITY = "extraBufferCapacity"
        const val TRY_EMIT = "tryEmit"
        const val BUFFER_ARGUMENT_POSITION = 2

        const val DESCRIPTION = "A SharedFlow fed by tryEmit has an explicit buffer."
    }
}
