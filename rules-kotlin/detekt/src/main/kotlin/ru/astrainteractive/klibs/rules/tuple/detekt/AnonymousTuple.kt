package ru.astrainteractive.klibs.rules.tuple.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.RequiresAnalysisApi
import dev.detekt.api.Rule
import org.jetbrains.kotlin.analysis.api.analyze
import org.jetbrains.kotlin.analysis.api.resolution.singleFunctionCallOrNull
import org.jetbrains.kotlin.analysis.api.resolution.symbol
import org.jetbrains.kotlin.analysis.api.symbols.KaClassLikeSymbol
import org.jetbrains.kotlin.analysis.api.symbols.KaConstructorSymbol
import org.jetbrains.kotlin.analysis.api.symbols.KaTypeAliasSymbol
import org.jetbrains.kotlin.analysis.api.types.KaErrorType
import org.jetbrains.kotlin.analysis.api.types.symbol
import org.jetbrains.kotlin.idea.references.mainReference
import org.jetbrains.kotlin.psi.KtBinaryExpression
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtUserType
import org.jetbrains.kotlin.psi.KtValueArgument
import org.jetbrains.kotlin.psi.psiUtil.getStrictParentOfType

/**
 * Resolves Kotlin's `Pair` and `Triple` types and constructors, including aliases, and pairs inferred from `a to b`.
 * Direct entries of standard map builders are allowed; unrelated DSL operators and names are not tuples.
 */
internal class AnonymousTuple(config: Config) : Rule(config, DESCRIPTION), RequiresAnalysisApi {

    private fun reportTuple(entity: Entity, name: String) {
        report(Finding(entity, "Replace $name with a data class whose properties are named."))
    }

    private fun isBuilderEntry(expression: KtBinaryExpression): Boolean {
        val argument = expression.parent as? KtValueArgument ?: return false
        val call = argument.getStrictParentOfType<KtCallExpression>() ?: return false
        return analyze(call) {
            call.resolveToCall()?.singleFunctionCallOrNull()?.symbol?.callableId?.asSingleFqName()?.asString() in
                MAP_BUILDERS
        }
    }

    override fun visitUserType(type: KtUserType) {
        super.visitUserType(type)
        val name = analyze(type) {
            val symbol = type.referenceExpression?.mainReference?.resolveToSymbol()
            val classId = if (symbol is KaTypeAliasSymbol) {
                symbol.expandedType.symbol?.classId
            } else {
                (symbol as? KaClassLikeSymbol)?.classId
            }
            classId?.asFqNameString()
        }?.takeIf(TUPLE_NAMES::contains)?.substringAfterLast('.') ?: return
        reportTuple(Entity.from(type), name)
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        val name = analyze(expression) {
            val call = expression.resolveToCall()?.singleFunctionCallOrNull()
            if (call?.symbol !is KaConstructorSymbol) return@analyze null
            expression.expressionType?.symbol?.classId?.asFqNameString()
        }
            ?.takeIf(TUPLE_NAMES::contains)?.substringAfterLast('.') ?: return
        reportTuple(Entity.from(expression), name)
    }

    override fun visitBinaryExpression(expression: KtBinaryExpression) {
        super.visitBinaryExpression(expression)
        if (expression.operationReference.getReferencedName() != TO) return
        if (isBuilderEntry(expression)) return
        val type = analyze(expression) {
            val left = expression.left?.expressionType ?: return@analyze null
            val right = expression.right?.expressionType ?: return@analyze null
            if (left is KaErrorType || right is KaErrorType) return@analyze null
            expression.expressionType?.symbol?.classId?.asFqNameString()
        }
        if (type != "kotlin.Pair") return
        reportTuple(Entity.from(expression), PAIR)
    }

    private companion object {
        const val TO = "to"
        const val PAIR = "Pair"

        @JvmField
        val TUPLE_NAMES = setOf("kotlin.Pair", "kotlin.Triple")

        @JvmField
        val MAP_BUILDERS = setOf(
            "kotlin.collections.mapOf",
            "kotlin.collections.mutableMapOf",
            "kotlin.collections.hashMapOf",
            "kotlin.collections.linkedMapOf",
            "kotlin.collections.sortedMapOf"
        )
        const val DESCRIPTION = "Typed data is a data class with named properties, never Pair or Triple."
    }
}
