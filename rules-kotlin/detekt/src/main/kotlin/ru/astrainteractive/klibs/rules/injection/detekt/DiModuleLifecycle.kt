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
import org.jetbrains.kotlin.analysis.api.types.KaType
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
import dev.detekt.api.config as ruleConfig

/**
 * In projects using the configured lifecycle convention, a module exposes the lifecycle of resources it directly
 * constructs, including construction deferred by `lazy`. The convention is not mandatory for every DI module.
 * Injected modules and their aliases remain the provider's responsibility. Unknown factories give no ownership
 * evidence; a service-like class name alone gives no lifecycle evidence.
 */
internal class DiModuleLifecycle(config: Config) : Rule(config, DESCRIPTION), RequiresAnalysisApi {
    private val lifecycleTypes: List<String> by ruleConfig(LIFECYCLE_TYPES)

    private fun KaType.isModuleLifecycle(element: KtClassOrObject): Boolean {
        return analyze(element) {
            lifecycleTypes.any { name -> isSubtypeOf(ClassId.topLevel(FqName(name))) }
        }
    }

    private fun isModule(module: KtClassOrObject): Boolean {
        return analyze(module) {
            val symbol = module.classSymbol ?: return@analyze false
            if (symbol.defaultType.isModuleLifecycle(module)) return@analyze false
            val namedModule = module.name?.endsWith(MODULE_SUFFIX) == true
            val implementsModule = symbol.superTypes.any { type ->
                type.symbol?.name?.asString()?.endsWith(MODULE_SUFFIX) == true
            }
            namedModule || implementsModule
        }
    }

    private fun hasLifecycleProperty(call: KtCallExpression, module: KtClassOrObject): Boolean {
        return analyze(call) {
            val type = call.expressionType ?: return@analyze false
            (type.symbol as? KaClassSymbol)?.memberScope?.callables.orEmpty()
                .filterIsInstance<KaPropertySymbol>()
                .any { member -> member.name.asString() == LIFECYCLE && member.returnType.isModuleLifecycle(module) }
        }
    }

    private fun usesLifecycleConvention(module: KtClassOrObject): Boolean {
        val imported = module.containingKtFile.importDirectives.mapNotNull { directive -> directive.importedFqName }
        val importedLifecycle = analyze(module) {
            lifecycleTypes.any { name ->
                val fqName = FqName(name)
                fqName in imported && findClass(ClassId.topLevel(fqName)) != null
            }
        }
        if (importedLifecycle) return true
        return module.declarations.filterIsInstance<KtProperty>()
            .mapNotNull(KtProperty::ownedConstructionOrNull)
            .any { call -> hasLifecycleProperty(call, module) }
    }

    private fun isLifecycleResource(call: KtCallExpression): Boolean {
        return analyze(call) {
            val type = call.expressionType ?: return@analyze false
            val module = call.getStrictParentOfType<KtClassOrObject>() ?: return@analyze false
            val hasLifecycle = hasLifecycleProperty(call, module)
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
        if (!isModule(module) || !usesLifecycleConvention(module)) return
        val actualLifecycle = analyze(expression) { expression.expressionType?.isModuleLifecycle(module) == true }
        if (!actualLifecycle) return
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
        if (!isModule(classOrObject) || !usesLifecycleConvention(classOrObject)) return
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
        if (!isModule(owner) || !usesLifecycleConvention(owner) || function.name !in LIFECYCLE_NAMES) return
        val message = "Remove '${function.name}' from ${owner.name}: expose val lifecycle: Lifecycle and let " +
            "the parent module collect it."
        report(Finding(Entity.atName(function), message))
    }

    private companion object {
        const val MODULE_SUFFIX = "Module"
        const val LIFECYCLE = "lifecycle"

        @JvmField
        val LIFECYCLE_TYPES = listOf(
            "ru.astrainteractive.astralibs.lifecycle.Lifecycle",
            "ru.astrainteractive.astralibs.lifecycle.api.Lifecycle"
        )

        @JvmField
        val LIFECYCLE_NAMES = setOf(
            "enable",
            "disable",
            "reload",
            "onEnable",
            "onDisable",
            "onReload"
        )
        const val DESCRIPTION = "Modules using the lifecycle convention expose owned lifecycles, not borrowed ones."
    }
}
