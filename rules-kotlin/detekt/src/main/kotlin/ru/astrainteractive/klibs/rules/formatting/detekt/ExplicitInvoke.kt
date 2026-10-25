package ru.astrainteractive.klibs.rules.formatting.detekt

import com.intellij.psi.PsiElement
import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtBlockExpression
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtClass
import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtFile
import org.jetbrains.kotlin.psi.KtFunctionLiteral
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtParameter
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtQualifiedExpression
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType
import org.jetbrains.kotlin.psi.psiUtil.parents

/**
 * Reports a call `name(…)` without a receiver where `name` is a value: a parameter of an enclosing function or
 * lambda, a constructor parameter or property of an enclosing class, or a local `val`/`var` declared above the
 * call. A function of that name declared anywhere in the file wins, so its calls are left alone.
 */
internal class ExplicitInvoke(config: Config) : Rule(config, DESCRIPTION) {
    private var functionNames: Set<String> = emptySet()

    private fun localNamesAbove(block: KtBlockExpression, child: PsiElement): List<String?> {
        return block.statements
            .takeWhile { statement -> statement != child }
            .filterIsInstance<KtProperty>()
            .map(KtProperty::getName)
    }

    private fun scopeNames(scope: PsiElement, child: PsiElement): List<String?> {
        return when (scope) {
            is KtBlockExpression -> localNamesAbove(scope, child)
            is KtFunctionLiteral -> scope.valueParameters.map(KtParameter::getName)
            is KtNamedFunction -> scope.valueParameters.map(KtParameter::getName)
            is KtClass -> scope.primaryConstructorParameters.map(KtParameter::getName) +
                scope.getProperties().map(KtProperty::getName)
            is KtClassOrObject -> scope.declarations.filterIsInstance<KtProperty>().map(KtProperty::getName)
            else -> emptyList()
        }
    }

    private fun isValueInScope(call: KtCallExpression, name: String): Boolean {
        var child: PsiElement = call
        for (scope in call.parents) {
            if (name in scopeNames(scope, child)) return true
            child = scope
        }
        return false
    }

    override fun visitKtFile(file: KtFile) {
        functionNames = file.collectDescendantsOfType<KtNamedFunction>()
            .mapNotNull(KtNamedFunction::getName)
            .toSet()
        super.visitKtFile(file)
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        val callee = expression.calleeExpression as? KtNameReferenceExpression ?: return
        val qualified = expression.parent as? KtQualifiedExpression
        if (qualified?.selectorExpression == expression) return
        val name = callee.getReferencedName()
        if (name in functionNames) return
        if (!isValueInScope(expression, name)) return
        val message = "'$name' is a value: call it as '$name.invoke(…)' so the reader sees a value being called."
        report(Finding(Entity.from(expression), message))
    }

    private companion object {
        const val DESCRIPTION = "A call on a property or a parameter is written with .invoke()."
    }
}
