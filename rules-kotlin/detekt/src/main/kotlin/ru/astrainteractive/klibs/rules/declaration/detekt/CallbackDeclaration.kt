package ru.astrainteractive.klibs.rules.declaration.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtCallableDeclaration
import org.jetbrains.kotlin.psi.KtClass
import org.jetbrains.kotlin.psi.KtFunctionType
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtNullableType
import org.jetbrains.kotlin.psi.KtParameter
import org.jetbrains.kotlin.psi.KtProperty
import ru.astrainteractive.klibs.rules.detekt.util.isTestSource

/**
 * Reports a callback the project declares in its own API: a parameter or property `on…` of a function type outside a
 * `@Composable` function, an `…Listener` interface, and an `…Callback` interface that is not the `fun interface
 * On<Event>Callback` of a Decompose component output. A result that arrives later is a `suspend` function, things
 * that happen are a `Flow`. An overridden declaration and the test sources are not judged.
 */
internal class CallbackDeclaration(config: Config) : Rule(config, DESCRIPTION) {

    private fun isFunctionTyped(declaration: KtCallableDeclaration): Boolean {
        val typeElement = declaration.typeReference?.typeElement
        val type = (typeElement as? KtNullableType)?.innerType ?: typeElement
        return type is KtFunctionType
    }

    private fun isCallbackValue(declaration: KtCallableDeclaration): Boolean {
        if (declaration.containingKtFile.isTestSource()) return false
        if (declaration.hasModifier(KtTokens.OVERRIDE_KEYWORD)) return false
        return CALLBACK_NAME.matches(declaration.name.orEmpty()) && isFunctionTyped(declaration)
    }

    private fun reportValue(declaration: KtCallableDeclaration) {
        val message = "Replace the callback '${declaration.name}' with a suspend function for a result or a Flow for " +
            "events; only a @Composable takes on… lambdas."
        report(Finding(Entity.atName(declaration), message))
    }

    override fun visitParameter(parameter: KtParameter) {
        super.visitParameter(parameter)
        if (!isCallbackValue(parameter)) return
        val owner = parameter.ownerFunction as? KtNamedFunction
        if (owner?.hasAnnotation(COMPOSABLE) == true) return
        reportValue(parameter)
    }

    override fun visitProperty(property: KtProperty) {
        super.visitProperty(property)
        if (!property.isLocal && isCallbackValue(property)) reportValue(property)
    }

    override fun visitClass(klass: KtClass) {
        super.visitClass(klass)
        if (!klass.isInterface() || klass.containingKtFile.isTestSource()) return
        val name = klass.name.orEmpty()
        val isDecomposeOutput = klass.hasModifier(KtTokens.FUN_KEYWORD) && OUTPUT_CALLBACK_NAME.matches(name)
        val isListener = name.endsWith(LISTENER_SUFFIX)
        val isCallback = name.endsWith(CALLBACK_SUFFIX) && !isDecomposeOutput
        if (!isListener && !isCallback) return
        val message = "Replace the interface '$name' with a Flow of a sealed event type from model; a listener stays " +
            "only when the producer must wait for its handlers, run them in order or read their result."
        report(Finding(Entity.atName(klass), message))
    }

    private companion object {
        const val COMPOSABLE = "Composable"
        const val LISTENER_SUFFIX = "Listener"
        const val CALLBACK_SUFFIX = "Callback"

        @JvmField
        val CALLBACK_NAME = Regex("on[A-Z]\\w*")

        @JvmField
        val OUTPUT_CALLBACK_NAME = Regex("On[A-Z]\\w*Callback")
        const val DESCRIPTION = "The project declares no callback: a suspend function for a result, a Flow for events."
    }
}
