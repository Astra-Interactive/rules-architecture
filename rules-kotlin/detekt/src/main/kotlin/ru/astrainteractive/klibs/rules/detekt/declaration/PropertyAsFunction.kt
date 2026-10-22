package ru.astrainteractive.klibs.rules.detekt.declaration

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtBlockExpression
import org.jetbrains.kotlin.psi.KtClass
import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtReturnExpression
import org.jetbrains.kotlin.psi.KtThisExpression
import org.jetbrains.kotlin.psi.psiUtil.containingClassOrObject

/**
 * Reports a function with no parameters that only returns a property of its class or of its file
 * (`fun foo() = bar`, `fun foo(): X { return bar }`): a stored value is a `val`. Overrides, `suspend` and
 * `@Composable` functions are skipped, and so are extensions, whose receiver is not the class.
 */
internal class PropertyAsFunction(config: Config) : Rule(config, DESCRIPTION) {

    private fun returnedExpression(function: KtNamedFunction): KtExpression? {
        val body = function.bodyExpression
        if (body !is KtBlockExpression) return body
        val statement = body.statements.singleOrNull() as? KtReturnExpression
        return statement?.returnedExpression
    }

    private fun referencedName(expression: KtExpression?): String? {
        if (expression is KtNameReferenceExpression) return expression.getReferencedName()
        if (expression !is KtDotQualifiedExpression) return null
        if (expression.receiverExpression !is KtThisExpression) return null
        return (expression.selectorExpression as? KtNameReferenceExpression)?.getReferencedName()
    }

    private fun propertyNames(function: KtNamedFunction): Set<String> {
        val owner: KtClassOrObject? = function.containingClassOrObject
        val declarations = owner?.declarations ?: function.containingKtFile.declarations
        val bodyNames = declarations.filterIsInstance<KtProperty>().mapNotNull { property -> property.name }
        val constructorNames = (owner as? KtClass)?.primaryConstructorParameters.orEmpty()
            .filter { parameter -> parameter.hasValOrVar() }
            .mapNotNull { parameter -> parameter.name }
        return (bodyNames + constructorNames).toSet()
    }

    private fun isExempt(function: KtNamedFunction): Boolean {
        if (function.valueParameters.isNotEmpty() || function.receiverTypeReference != null) return true
        if (function.hasModifier(KtTokens.OVERRIDE_KEYWORD)) return true
        if (function.hasModifier(KtTokens.SUSPEND_KEYWORD)) return true
        return function.hasAnnotation(COMPOSABLE)
    }

    override fun visitNamedFunction(function: KtNamedFunction) {
        super.visitNamedFunction(function)
        if (isExempt(function)) return
        val name = referencedName(returnedExpression(function)) ?: return
        if (name !in propertyNames(function)) return
        val message = "'${function.name}' only returns the property '$name': make it a val."
        report(Finding(Entity.atName(function), message))
    }

    private companion object {
        const val COMPOSABLE = "Composable"
        const val DESCRIPTION = "A function with no arguments that only returns a stored value is a val."
    }
}
