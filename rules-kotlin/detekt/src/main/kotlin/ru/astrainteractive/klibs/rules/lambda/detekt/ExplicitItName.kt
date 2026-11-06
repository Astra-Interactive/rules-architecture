package ru.astrainteractive.klibs.rules.lambda.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtDestructuringDeclarationEntry
import org.jetbrains.kotlin.psi.KtExpressionWithLabel
import org.jetbrains.kotlin.psi.KtForExpression
import org.jetbrains.kotlin.psi.KtLabeledExpression
import org.jetbrains.kotlin.psi.KtNamedDeclaration
import org.jetbrains.kotlin.psi.KtParameter
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.psiUtil.parents

/**
 * Reports a declaration named `it` on its name, a label `it@` included, and a jump to `@it` that no enclosing `it@`
 * declares (the label of a call, a function or a class named `it`).
 *
 * An `override` and a parameter of an `override` function pass: the supertype's declaration is the one reported. The
 * implicit `it` is reported by [ImplicitItParameter].
 */
internal class ExplicitItName(config: Config) : Rule(config, DESCRIPTION) {

    private fun selectKind(declaration: KtNamedDeclaration): String {
        return when {
            declaration is KtDestructuringDeclarationEntry -> "entry"
            declaration is KtProperty -> "property"
            declaration.parent is KtForExpression -> "loop variable"
            declaration is KtParameter -> "parameter"
            else -> "declaration"
        }
    }

    private fun isOverride(declaration: KtNamedDeclaration): Boolean {
        if (declaration.hasModifier(KtTokens.OVERRIDE_KEYWORD)) return true
        val owner = (declaration as? KtParameter)?.ownerFunction ?: return false
        return owner.hasModifier(KtTokens.OVERRIDE_KEYWORD)
    }

    private fun hasEnclosingLabel(jump: KtExpressionWithLabel): Boolean {
        return jump.parents.any { parent -> parent is KtLabeledExpression && parent.getLabelName() == FORBIDDEN_NAME }
    }

    private fun reportName(entity: Entity, kind: String) {
        report(Finding(entity, "Name the $kind instead of `it`."))
    }

    override fun visitNamedDeclaration(declaration: KtNamedDeclaration) {
        super.visitNamedDeclaration(declaration)
        if (declaration.name != FORBIDDEN_NAME) return
        if (isOverride(declaration)) return
        reportName(Entity.atName(declaration), selectKind(declaration))
    }

    override fun visitExpressionWithLabel(expression: KtExpressionWithLabel) {
        super.visitExpressionWithLabel(expression)
        val label = expression.getTargetLabel() ?: return
        if (label.getReferencedName() != FORBIDDEN_NAME) return
        if (expression !is KtLabeledExpression && hasEnclosingLabel(expression)) return
        reportName(Entity.from(label.getReferencedNameElement()), "label")
    }

    private companion object {
        const val FORBIDDEN_NAME = "it"
        const val DESCRIPTION = "Nothing is named `it`: parameters, properties, destructured entries, loop variables " +
            "and labels carry explicit, meaningful names."
    }
}
