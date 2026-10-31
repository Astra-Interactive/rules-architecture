package ru.astrainteractive.klibs.rules.injection.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.name.Name
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtClass
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports a constructor call anywhere in `RootModule`, properties, constructor bodies and functions included, when
 * it constructs something other than a DI module or a `Lifecycle`: the composition root builds the root DI modules
 * and lists their lifecycles, and constructs no service itself. A constructor call starts with an upper-case letter.
 * A DI module is a class declared in a `di` package: the package comes from the import of the call's first name
 * (`CoreModule` in `CoreModule.Default()`), from a fully qualified call, or else from the file's own package.
 */
internal class RootModuleConstruction(config: Config) : Rule(config, DESCRIPTION) {

    private fun callChain(call: KtCallExpression): List<String> {
        val callee = call.calleeExpression?.text.orEmpty()
        val qualified = call.parent as? KtDotQualifiedExpression
        if (qualified?.selectorExpression != call) return listOf(callee)
        return qualified.receiverExpression.text.split('.') + callee
    }

    private fun packageOf(call: KtCallExpression, chain: List<String>): List<String> {
        val qualifiedPackage = chain.takeWhile { segment -> segment.first().isLowerCase() }
        if (qualifiedPackage.isNotEmpty()) return qualifiedPackage
        val file = call.containingKtFile
        val import = file.importDirectives.firstOrNull { directive ->
            directive.importedName?.asString() == chain.first()
        }
        val packageName = import?.importedFqName?.parent() ?: file.packageFqName
        return packageName.pathSegments().map(Name::asString)
    }

    private fun isAllowed(call: KtCallExpression): Boolean {
        val chain = callChain(call)
        val name = chain.last()
        if (name.endsWith(LIFECYCLE)) return true
        if (name == LAMBDA && chain.dropLast(1).lastOrNull() == LIFECYCLE) return true
        return DI_SEGMENT in packageOf(call, chain)
    }

    override fun visitClass(klass: KtClass) {
        super.visitClass(klass)
        if (klass.name != ROOT_MODULE) return
        klass.collectDescendantsOfType<KtCallExpression>()
            .filter { call -> call.calleeExpression?.text.orEmpty().firstOrNull()?.isUpperCase() == true }
            .filterNot(::isAllowed)
            .forEach { call ->
                val message = "RootModule constructs '${call.calleeExpression?.text}': construct it in the DI " +
                    "module that owns it and pass that module; RootModule only builds the root DI modules."
                report(Finding(Entity.from(call), message))
            }
    }

    private companion object {
        const val ROOT_MODULE = "RootModule"
        const val LIFECYCLE = "Lifecycle"
        const val LAMBDA = "Lambda"
        const val DI_SEGMENT = "di"
        const val DESCRIPTION = "RootModule only builds the root DI modules and lists their lifecycles."
    }
}
