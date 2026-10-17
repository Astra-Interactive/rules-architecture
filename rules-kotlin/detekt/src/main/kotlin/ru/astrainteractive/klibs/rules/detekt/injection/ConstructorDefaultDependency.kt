package ru.astrainteractive.klibs.rules.detekt.injection

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtClass
import org.jetbrains.kotlin.psi.KtConstructor
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtPrimaryConstructor
import org.jetbrains.kotlin.psi.KtQualifiedExpression
import org.jetbrains.kotlin.psi.KtSecondaryConstructor

/**
 * Reports a constructor parameter whose default value constructs an object, such as `= Foo()` or `= foo.Bar()`.
 *
 * Works on syntax only: a call counts as a construction when its callee starts with an upper-case letter, so a
 * factory function (`= createFoo()`) and an object reference (`= Clock.System`) are not seen. A `@Serializable`
 * class is exempt, because serialization needs its defaults; other data classes are left to
 * [ru.astrainteractive.klibs.rules.detekt.declaration.DataClassDefaultValue].
 */
internal class ConstructorDefaultDependency(config: Config) : Rule(config, DESCRIPTION) {

    private fun isConstruction(expression: KtExpression?): Boolean {
        val call = when (expression) {
            is KtCallExpression -> expression
            is KtQualifiedExpression -> expression.selectorExpression as? KtCallExpression
            else -> null
        }
        val callee = call?.calleeExpression as? KtNameReferenceExpression ?: return false
        return callee.getReferencedName().first().isUpperCase()
    }

    private fun checkConstructor(constructor: KtConstructor<*>) {
        val owner = constructor.getContainingClassOrObject()
        if (owner is KtClass && owner.isData()) return
        if (owner.hasAnnotation(SERIALIZABLE)) return
        constructor.valueParameters
            .filter { parameter -> isConstruction(parameter.defaultValue) }
            .forEach { parameter ->
                val message = "Construct '${parameter.name}' in the DI module and pass it in, " +
                    "instead of creating it in the constructor default."
                report(Finding(Entity.from(parameter), message))
            }
    }

    override fun visitPrimaryConstructor(constructor: KtPrimaryConstructor) {
        super.visitPrimaryConstructor(constructor)
        checkConstructor(constructor)
    }

    override fun visitSecondaryConstructor(constructor: KtSecondaryConstructor) {
        super.visitSecondaryConstructor(constructor)
        checkConstructor(constructor)
    }

    private companion object {
        const val SERIALIZABLE = "Serializable"
        const val DESCRIPTION = "Dependencies are constructed at the composition root, never in a constructor default."
    }
}
