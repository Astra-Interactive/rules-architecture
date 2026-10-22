package ru.astrainteractive.klibs.rules.detekt.injection

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtClass
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports a property of `RootModule` whose initializer, a `lazy { }` one included, constructs something other than
 * a DI module or a `Lifecycle`: the composition root builds the root DI modules and lists their lifecycles, and
 * constructs no service itself. A constructor call is a call whose name starts with an upper-case letter.
 */
internal class RootModuleConstruction(config: Config) : Rule(config, DESCRIPTION) {

    private fun isAllowed(call: KtCallExpression, name: String): Boolean {
        if (ALLOWED_SUFFIXES.any { suffix -> name.endsWith(suffix) }) return true
        val qualified = call.parent as? KtDotQualifiedExpression ?: return false
        return name == LAMBDA && qualified.receiverExpression.text == LIFECYCLE
    }

    private fun constructedNames(property: KtProperty): List<KtCallExpression> {
        val initializers = listOfNotNull(property.initializer, property.delegateExpression)
        return initializers
            .flatMap { initializer -> initializer.collectDescendantsOfType<KtCallExpression>() }
            .filter { call -> call.calleeExpression?.text.orEmpty().firstOrNull()?.isUpperCase() == true }
            .filterNot { call -> isAllowed(call, call.calleeExpression?.text.orEmpty()) }
    }

    override fun visitClass(klass: KtClass) {
        super.visitClass(klass)
        if (klass.name != ROOT_MODULE) return
        klass.getProperties()
            .flatMap(::constructedNames)
            .forEach { call ->
                val message = "RootModule constructs '${call.calleeExpression?.text}': construct it in the DI " +
                    "module that owns it and pass that module; RootModule only builds the root DI modules."
                report(Finding(Entity.from(call), message))
            }
    }

    private companion object {
        const val ROOT_MODULE = "RootModule"
        const val LIFECYCLE = "Lifecycle"
        const val LAMBDA = "Lambda"

        @JvmField
        val ALLOWED_SUFFIXES = listOf("Module", "Lifecycle")
        const val DESCRIPTION = "RootModule only builds the root DI modules and lists their lifecycles."
    }
}
