package ru.astrainteractive.klibs.rules.tuple.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtBinaryExpression
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtUserType
import org.jetbrains.kotlin.psi.KtValueArgument
import org.jetbrains.kotlin.psi.psiUtil.getStrictParentOfType

/**
 * Reports `Pair` and `Triple` written as a type, constructed by name, or inferred from `a to b`; `a to b` is allowed
 * only as an entry of `mapOf(…)` and the other map builders.
 */
internal class AnonymousTuple(config: Config) : Rule(config, DESCRIPTION) {

    private fun reportTuple(entity: Entity, name: String) {
        report(Finding(entity, "Replace $name with a data class whose properties are named."))
    }

    private fun isBuilderEntry(expression: KtBinaryExpression): Boolean {
        val argument = expression.parent as? KtValueArgument ?: return false
        val call = argument.getStrictParentOfType<KtCallExpression>() ?: return false
        return call.calleeExpression?.text in MAP_BUILDERS
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

    override fun visitBinaryExpression(expression: KtBinaryExpression) {
        super.visitBinaryExpression(expression)
        if (expression.operationReference.getReferencedName() != TO) return
        if (isBuilderEntry(expression)) return
        reportTuple(Entity.from(expression), PAIR)
    }

    private companion object {
        const val TO = "to"
        const val PAIR = "Pair"

        @JvmField
        val TUPLE_NAMES = setOf(PAIR, "Triple")

        @JvmField
        val MAP_BUILDERS = setOf(
            "mapOf",
            "mutableMapOf",
            "hashMapOf",
            "linkedMapOf",
            "sortedMapOf"
        )
        const val DESCRIPTION = "Typed data is a data class with named properties, never Pair or Triple."
    }
}
