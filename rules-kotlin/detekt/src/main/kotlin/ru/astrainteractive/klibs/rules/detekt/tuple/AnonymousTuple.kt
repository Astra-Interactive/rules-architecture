package ru.astrainteractive.klibs.rules.detekt.tuple

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtUserType

/**
 * Reports `Pair` and `Triple` written as a type or constructed by name.
 *
 * Works on syntax only, so a tuple whose type is inferred (`val entry = key to value`) is not seen.
 */
internal class AnonymousTuple(config: Config) : Rule(config, DESCRIPTION) {

    private fun reportTuple(entity: Entity, name: String) {
        report(Finding(entity, "Replace $name with a data class whose properties are named."))
    }

    override fun visitUserType(type: KtUserType) {
        super.visitUserType(type)
        val name = type.referencedName ?: return
        if (name !in TUPLE_NAMES) return
        reportTuple(Entity.from(type), name)
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        val callee = expression.calleeExpression as? KtNameReferenceExpression ?: return
        val name = callee.getReferencedName()
        if (name !in TUPLE_NAMES) return
        reportTuple(Entity.from(expression), name)
    }

    private companion object {
        @JvmField
        val TUPLE_NAMES = setOf("Pair", "Triple")
        const val DESCRIPTION = "Typed data is a data class with named properties, never Pair or Triple."
    }
}
