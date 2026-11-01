package ru.astrainteractive.klibs.rules.injection.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.RequiresAnalysisApi
import dev.detekt.api.Rule
import org.jetbrains.kotlin.analysis.api.analyze
import org.jetbrains.kotlin.analysis.api.resolution.singleFunctionCallOrNull
import org.jetbrains.kotlin.analysis.api.resolution.symbol
import org.jetbrains.kotlin.analysis.api.symbols.KaClassKind
import org.jetbrains.kotlin.analysis.api.symbols.KaConstructorSymbol
import org.jetbrains.kotlin.analysis.api.symbols.KaNamedClassSymbol
import org.jetbrains.kotlin.analysis.api.symbols.KaNamedFunctionSymbol
import org.jetbrains.kotlin.analysis.api.types.symbol
import org.jetbrains.kotlin.idea.references.mainReference
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtClass
import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtParameter
import org.jetbrains.kotlin.psi.KtParenthesizedExpression
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtReturnExpression
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType
import org.jetbrains.kotlin.psi.psiUtil.getStrictParentOfType
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.detekt.util.unitContextOrNull

/**
 * Resolves actual constructors and distinguishes retained/called dependencies from returned instances and owned
 * state. Unknown calls are not guessed from capitalized names. Constructor defaults have their own rule.
 */
internal class FieldDependencyConstruction(config: Config) : Rule(config, DESCRIPTION), RequiresAnalysisApi {
    private fun isValueOrState(symbol: KaNamedClassSymbol): Boolean {
        if (symbol.isData || symbol.isInline || symbol.classKind == KaClassKind.ENUM_CLASS) return true
        return symbol.classId?.asSingleFqName()?.asString() in VALUE_AND_STATE_TYPES
    }

    private fun isStateOperation(call: KtCallExpression, owner: KtClass): Boolean {
        return analyze(call) {
            val function = call.resolveToCall()?.singleFunctionCallOrNull()?.symbol ?: return@analyze false
            if (function is KaConstructorSymbol) {
                val type = function.returnType.symbol as? KaNamedClassSymbol ?: return@analyze false
                return@analyze isValueOrState(type)
            }
            val id = function.callableId ?: return@analyze false
            if (id.classId?.asSingleFqName() == owner.fqName) return@analyze true
            val packageName = id.packageName.asString()
            packageName in STATE_OPERATION_PACKAGES || id.classId?.asSingleFqName()?.asString() in VALUE_AND_STATE_TYPES
        }
    }

    private fun isOwnedState(symbol: KaNamedClassSymbol): Boolean {
        val source = symbol.psi as? KtClass ?: return false
        val properties = source.declarations.filterIsInstance<KtProperty>()
        val mutableState = properties.any { property ->
            if (property.isVar) return@any true
            analyze(property) {
                property.returnType.symbol?.classId?.asSingleFqName()?.asString() in MUTABLE_STATE_TYPES
            }
        }
        if (!mutableState) return false
        return source.collectDescendantsOfType<KtCallExpression>().all { call -> isStateOperation(call, source) }
    }

    private fun dependencyName(call: KtCallExpression): String? {
        return analyze(call) {
            val constructor = call.resolveToCall()?.singleFunctionCallOrNull()?.symbol as? KaConstructorSymbol
                ?: return@analyze null
            val symbol = constructor.returnType.symbol as? KaNamedClassSymbol ?: return@analyze null
            if (isValueOrState(symbol) || isOwnedState(symbol)) return@analyze null
            val entryPoint = config.unitContextOrNull()?.kind == UnitKind.ENTRY_POINT
            val inDi = symbol.classId?.packageFqName?.pathSegments()
                ?.any { segment -> segment.asString() == "di" } == true
            if (entryPoint && inDi) return@analyze null
            val behavior = symbol.memberScope.callables.filterIsInstance<KaNamedFunctionSymbol>()
                .any { function -> function.callableId?.classId?.asSingleFqName()?.asString() != "kotlin.Any" }
            if (!behavior) return@analyze null
            symbol.name.asString()
        }
    }

    private fun isReturnedValue(expression: KtExpression, seen: Set<KtProperty> = emptySet()): Boolean {
        val parent = expression.parent
        if (parent is KtParenthesizedExpression) return isReturnedValue(parent, seen)
        if (parent is KtDotQualifiedExpression && parent.selectorExpression == expression) {
            return isReturnedValue(parent, seen)
        }
        if (parent is KtReturnExpression) return parent.returnedExpression == expression
        if (parent is KtNamedFunction) return !parent.hasBlockBody() && parent.bodyExpression == expression
        if (parent !is KtProperty || !parent.isLocal || parent in seen) return false
        val function = parent.getStrictParentOfType<KtNamedFunction>() ?: return false
        val references = function.collectDescendantsOfType<KtNameReferenceExpression>().filter { reference ->
            analyze(reference) { reference.mainReference.resolveToSymbol()?.psi == parent }
        }
        return references.isNotEmpty() && references.all { reference -> isReturnedValue(reference, seen + parent) }
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        val inDi = expression.containingKtFile.packageFqName.pathSegments()
            .any { segment -> segment.asString() == "di" }
        if (inDi || expression.getStrictParentOfType<KtParameter>() != null) return
        val owner = expression.getStrictParentOfType<KtClassOrObject>() ?: return
        if (isReturnedValue(expression)) return
        val dependency = dependencyName(expression) ?: return
        val message = "${owner.name} constructs $dependency and retains it or uses its behavior outside di: " +
            "construct the dependency in DI and pass it through the constructor. Returned results and owned state " +
            "are not dependencies."
        report(Finding(Entity.from(expression), message))
    }

    private companion object {
        @JvmField
        val MUTABLE_STATE_TYPES = setOf(
            "kotlin.collections.MutableMap",
            "kotlin.collections.MutableList",
            "kotlin.collections.MutableSet",
            "java.util.HashMap",
            "java.util.ArrayList",
            "java.util.HashSet"
        )

        @JvmField
        val VALUE_AND_STATE_TYPES = MUTABLE_STATE_TYPES + setOf(
            "kotlin.String",
            "kotlin.Boolean",
            "kotlin.Int",
            "kotlin.Long",
            "kotlin.Float",
            "kotlin.Double",
            "kotlin.Byte",
            "kotlin.Short",
            "kotlin.Char",
            "java.lang.StringBuilder"
        )

        @JvmField
        val STATE_OPERATION_PACKAGES = setOf(
            "kotlin.collections",
            "kotlin.text",
            "kotlin.ranges"
        )
        const val DESCRIPTION = "Only concrete dependencies are constructed in DI; results and owned state stay local."
    }
}
