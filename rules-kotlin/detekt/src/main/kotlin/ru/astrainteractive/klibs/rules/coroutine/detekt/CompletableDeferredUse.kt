package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports a `CompletableDeferred` used where the no-callbacks rule forbids it: created and awaited in one function,
 * the replacement of a `suspend` function that `suspendCancellableCoroutine` bridges, and a `CompletableDeferred` of
 * `Unit` or `Boolean` held in a property as a readiness flag, which is a `StateFlow`.
 */
internal class CompletableDeferredUse(config: Config) : Rule(config, DESCRIPTION) {

    private fun isDeferredConstruction(property: KtProperty): Boolean {
        val initializer = property.initializer as? KtCallExpression
        return initializer?.calleeExpression?.text == COMPLETABLE_DEFERRED
    }

    private fun isFlagType(property: KtProperty): Boolean {
        val typeText = property.typeReference?.text ?: property.initializer?.text.orEmpty()
        return FLAG_TYPES.any { type -> "$COMPLETABLE_DEFERRED<$type>" in typeText }
    }

    private fun isAwaited(function: KtNamedFunction, name: String): Boolean {
        return function.collectDescendantsOfType<KtDotQualifiedExpression>().any { expression ->
            val call = expression.selectorExpression as? KtCallExpression
            expression.receiverExpression.text == name && call?.calleeExpression?.text == AWAIT
        }
    }

    override fun visitNamedFunction(function: KtNamedFunction) {
        super.visitNamedFunction(function)
        function.collectDescendantsOfType<KtProperty>()
            .filter { property -> property.isLocal && isDeferredConstruction(property) }
            .filter { property -> isAwaited(function, property.name.orEmpty()) }
            .forEach { property ->
                val message = "Bridge this result with suspendCancellableCoroutine instead of the " +
                    "CompletableDeferred '${property.name}': a deferred created and awaited in one function " +
                    "replaces a suspend function."
                report(Finding(Entity.from(property), message))
            }
    }

    override fun visitProperty(property: KtProperty) {
        super.visitProperty(property)
        if (property.isLocal || !isFlagType(property)) return
        val message = "Hold the readiness in a StateFlow instead of the CompletableDeferred '${property.name}': a " +
            "deferred is completed once and is no state."
        report(Finding(Entity.from(property), message))
    }

    private companion object {
        const val COMPLETABLE_DEFERRED = "CompletableDeferred"
        const val AWAIT = "await"

        @JvmField
        val FLAG_TYPES = listOf("Unit", "Boolean")
        const val DESCRIPTION = "A CompletableDeferred is completed elsewhere than it is awaited and is no flag."
    }
}
