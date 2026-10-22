package ru.astrainteractive.klibs.rules.detekt.ordering

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtClass
import org.jetbrains.kotlin.psi.KtClassBody
import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtConstantExpression
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtFile
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtObjectDeclaration
import org.jetbrains.kotlin.psi.KtPrefixExpression
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtStringTemplateExpression

/**
 * Reports class-level constants outside the companion object: a `const val`, or a `val` initialised with a
 * literal or a literal with one property or call after it (`1.minutes`, `"x".toRegex()`), declared in a class
 * body; and a top-level `private object` that only holds properties beside another type of the file.
 */
internal class ConstantOutsideCompanion(config: Config) : Rule(config, DESCRIPTION) {

    private fun isLiteral(expression: KtExpression?): Boolean {
        return when (expression) {
            is KtConstantExpression -> true
            is KtStringTemplateExpression -> !expression.hasInterpolation()
            is KtPrefixExpression -> {
                val isSign = expression.operationToken == KtTokens.MINUS
                isSign && expression.baseExpression is KtConstantExpression
            }
            else -> false
        }
    }

    private fun isConstantInitializer(expression: KtExpression?): Boolean {
        if (expression !is KtDotQualifiedExpression) return isLiteral(expression)
        val selector = expression.selectorExpression
        val isSingleSuffix = selector is KtNameReferenceExpression || selector is KtCallExpression
        return isSingleSuffix && isLiteral(expression.receiverExpression)
    }

    private fun isClassConstant(property: KtProperty): Boolean {
        val body = property.parent as? KtClassBody ?: return false
        if (body.parent !is KtClass) return false
        if (property.hasModifier(KtTokens.CONST_KEYWORD)) return true
        if (property.isVar || property.hasModifier(KtTokens.OVERRIDE_KEYWORD)) return false
        return isConstantInitializer(property.initializer)
    }

    private fun checkConstantObjects(file: KtFile) {
        val types = file.declarations.filterIsInstance<KtClassOrObject>()
        if (types.size < 2) return
        types.filterIsInstance<KtObjectDeclaration>()
            .filter { declaration -> declaration.hasModifier(KtTokens.PRIVATE_KEYWORD) }
            .filter { declaration -> declaration.declarations.isNotEmpty() }
            .filter { declaration -> declaration.declarations.all { member -> member is KtProperty } }
            .forEach { declaration ->
                val message = "Move the constants of '${declaration.name}' to the companion object of the type " +
                    "that uses them: a private object of constants beside a type is forbidden."
                report(Finding(Entity.atName(declaration), message))
            }
    }

    override fun visitProperty(property: KtProperty) {
        super.visitProperty(property)
        if (!isClassConstant(property)) return
        val message = "Move the constant '${property.name}' to the companion object: class-level constants live " +
            "there."
        report(Finding(Entity.atName(property), message))
    }

    override fun visitKtFile(file: KtFile) {
        super.visitKtFile(file)
        checkConstantObjects(file)
    }

    private companion object {
        const val DESCRIPTION = "Class-level constants live in the companion object, which is the last declaration."
    }
}
