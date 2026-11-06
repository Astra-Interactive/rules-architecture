package ru.astrainteractive.klibs.rules.lambda.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtDestructuringDeclarationEntry
import org.jetbrains.kotlin.psi.KtForExpression
import org.jetbrains.kotlin.psi.KtNamedDeclaration
import org.jetbrains.kotlin.psi.KtParameter
import org.jetbrains.kotlin.psi.KtProperty

/**
 * Reports every declaration named `it`, on its name: a parameter of a function, a constructor, a lambda, a function
 * type or a `catch`, a property, a destructured entry, a `for` loop variable, and any other named declaration.
 *
 * Works on syntax only. A use of a declared `it` is not reported, its declaration is; the implicit `it` is reported
 * by [ImplicitItParameter]. An `override` and a parameter of an `override` function pass: they keep the name their
 * supertype declares, and that declaration is the one reported.
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

    override fun visitNamedDeclaration(declaration: KtNamedDeclaration) {
        super.visitNamedDeclaration(declaration)
        if (declaration.name != FORBIDDEN_NAME) return
        if (isOverride(declaration)) return
        report(Finding(Entity.atName(declaration), "Name the ${selectKind(declaration)} instead of `it`."))
    }

    private companion object {
        const val FORBIDDEN_NAME = "it"
        const val DESCRIPTION = "Nothing is named `it`: parameters, properties, destructured entries and loop " +
            "variables carry explicit, meaningful names."
    }
}
