package ru.astrainteractive.klibs.rules.injection.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.RequiresAnalysisApi
import dev.detekt.api.Rule
import org.jetbrains.kotlin.analysis.api.analyze
import org.jetbrains.kotlin.analysis.api.resolution.singleFunctionCallOrNull
import org.jetbrains.kotlin.analysis.api.resolution.symbol
import org.jetbrains.kotlin.analysis.api.symbols.KaClassSymbol
import org.jetbrains.kotlin.analysis.api.symbols.KaPropertySymbol
import org.jetbrains.kotlin.analysis.api.types.symbol
import org.jetbrains.kotlin.idea.references.mainReference
import org.jetbrains.kotlin.name.ClassId
import org.jetbrains.kotlin.name.FqName
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtParameter
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType
import org.jetbrains.kotlin.psi.psiUtil.containingClassOrObject
import org.jetbrains.kotlin.psi.psiUtil.getStrictParentOfType
import ru.astrainteractive.klibs.rules.detekt.util.ownedConstructionOrNull

/**
 * A module exposes the lifecycle of resources it directly constructs, including construction deferred by `lazy`.
 * Injected modules and their aliases remain the provider's responsibility. Unknown factories give no ownership
 * evidence; a service-like class name alone gives no lifecycle evidence.
 */
internal class DiModuleLifecycle(config: Config) : Rule(config, DESCRIPTION), RequiresAnalysisApi {

    private fun isModule(module: KtClassOrObject): Boolean {
        if (module.name?.endsWith(MODULE_SUFFIX) == true) return true
        return module.containingKtFile.packageFqName.pathSegments().any { segment -> segment.asString() == DI_SEGMENT }
    }

    private fun isLifecycleResource(call: KtCallExpression): Boolean {
        return analyze(call) {
            val type = call.expressionType ?: return@analyze false
            val hasLifecycle = (type.symbol as? KaClassSymbol)?.memberScope?.callables.orEmpty()
                .filterIsInstance<KaPropertySymbol>()
                .any { member ->
                    member.name.asString() == LIFECYCLE && member.returnType.symbol?.name?.asString() == "Lifecycle"
                }
            if (hasLifecycle) return@analyze true
            val javaCloseable = type.isSubtypeOf(ClassId.topLevel(FqName("java.lang.AutoCloseable")))
            val kotlinCloseable = type.isSubtypeOf(ClassId.topLevel(FqName("kotlin.AutoCloseable")))
            javaCloseable || kotlinCloseable
        }
    }

    private fun findStartedName(module: KtClassOrObject): String? {
        val resource = module.declarations.filterIsInstance<KtProperty>()
            .mapNotNull(KtProperty::ownedConstructionOrNull)
            .firstOrNull(::isLifecycleResource)
        if (resource != null) return resource.calleeExpression?.text
        return module.collectDescendantsOfType<KtCallExpression>().firstOrNull { call ->
            if (call.getStrictParentOfType<KtClassOrObject>() != module) return@firstOrNull false
            analyze(call) {
                val symbol = call.resolveToCall()?.singleFunctionCallOrNull()?.symbol
                symbol?.callableId?.asSingleFqName()?.asString() == "kotlinx.coroutines.launch"
            }
        }?.calleeExpression?.text
    }

    private fun isBorrowed(expression: KtExpression, module: KtClassOrObject, seen: Set<KtProperty>): Boolean {
        val reference = when (expression) {
            is KtNameReferenceExpression -> expression
            is KtDotQualifiedExpression -> expression.selectorExpression as? KtNameReferenceExpression
            else -> null
        } ?: return false
        val declaration = analyze(reference) { reference.mainReference.resolveToSymbol()?.psi }
        if (declaration is KtParameter) return declaration.containingClassOrObject == module
        if (declaration !is KtProperty || declaration in seen) return false
        val value = declaration.initializer ?: return false
        return isBorrowed(
            value,
            module,
            seen.plus(declaration)
        )
    }

    override fun visitDotQualifiedExpression(expression: KtDotQualifiedExpression) {
        super.visitDotQualifiedExpression(expression)
        if ((expression.selectorExpression as? KtNameReferenceExpression)?.getReferencedName() != LIFECYCLE) return
        val module = expression.getStrictParentOfType<KtClassOrObject>() ?: return
        if (!isModule(module)) return
        val lifecycleProperty = expression.getStrictParentOfType<KtProperty>()?.name == LIFECYCLE
        val outer = expression.parent as? KtDotQualifiedExpression
        val transition = (outer?.selectorExpression as? KtCallExpression)?.calleeExpression?.text in LIFECYCLE_NAMES
        if (!lifecycleProperty && !transition) return
        val borrowed = isBorrowed(
            expression.receiverExpression,
            module,
            emptySet()
        )
        if (!borrowed) return
        val message = "Do not manage '${expression.text}' in ${module.name}: it belongs to an injected module; " +
            "only the module that constructs it manages its lifecycle."
        report(Finding(Entity.from(expression), message))
    }

    override fun visitClassOrObject(classOrObject: KtClassOrObject) {
        super.visitClassOrObject(classOrObject)
        val name = classOrObject.name ?: return
        if (!isModule(classOrObject)) return
        val hasLifecycle = classOrObject.declarations
            .filterIsInstance<KtProperty>()
            .any { property -> property.name == LIFECYCLE }
        if (hasLifecycle) return
        val started = findStartedName(classOrObject) ?: return
        val message = "Expose val lifecycle: Lifecycle from $name: it starts $started, and the parent module " +
            "starts and stops it through that lifecycle."
        report(Finding(Entity.atName(classOrObject), message))
    }

    override fun visitNamedFunction(function: KtNamedFunction) {
        super.visitNamedFunction(function)
        val owner = function.containingClassOrObject ?: return
        if (!isModule(owner) || function.name !in LIFECYCLE_NAMES) return
        val message = "Remove '${function.name}' from ${owner.name}: expose val lifecycle: Lifecycle and let " +
            "the parent module collect it."
        report(Finding(Entity.atName(function), message))
    }

    private companion object {
        const val MODULE_SUFFIX = "Module"
        const val LIFECYCLE = "lifecycle"
        const val DI_SEGMENT = "di"

        @JvmField
        val LIFECYCLE_NAMES = setOf(
            "enable",
            "disable",
            "reload",
            "onEnable",
            "onDisable",
            "onReload"
        )
        const val DESCRIPTION = "A DI module exposes val lifecycle: Lifecycle, never its own enable/disable/reload."
    }
}
