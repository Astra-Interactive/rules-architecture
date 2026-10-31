package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.RequiresAnalysisApi
import dev.detekt.api.Rule
import org.jetbrains.kotlin.analysis.api.analyze
import org.jetbrains.kotlin.analysis.api.symbols.KaPropertySymbol
import org.jetbrains.kotlin.idea.references.mainReference
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.name.Name
import org.jetbrains.kotlin.psi.KtImportDirective
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtPropertyAccessor
import org.jetbrains.kotlin.psi.KtReferenceExpression
import org.jetbrains.kotlin.psi.psiUtil.getStrictParentOfType
import ru.astrainteractive.klibs.rules.detekt.util.isTestSource
import dev.detekt.api.config as ruleConfig

/**
 * Reports a dispatcher read from `kotlinx.coroutines.Dispatchers` (`Dispatchers.IO`, an imported `IO`) where it
 * should be injected: a class takes its dispatchers through the constructor, from whatever interface the project
 * provides them with (`KotlinDispatchers`, its own), so a read through such a receiver is never reported. The read
 * is matched through the Analysis API, not by its text. A `di` package, the `override` property of a provider that
 * hands the dispatcher out (`override val IO = Dispatchers.IO`) and test sources are not judged.
 */
internal class HardcodedDispatcher(config: Config) : Rule(config, DESCRIPTION), RequiresAnalysisApi {
    private val dispatcherNames: List<String> by ruleConfig(listOf("IO", "Default", "Unconfined"))

    private fun isKotlinxDispatcher(expression: KtNameReferenceExpression): Boolean {
        val callableId = analyze(expression) {
            (expression.mainReference.resolveToSymbol() as? KaPropertySymbol)?.callableId
        } ?: return false
        val isCoroutinesPackage = callableId.packageName.asString() == COROUTINES_PACKAGE
        return isCoroutinesPackage && callableId.className?.asString() == DISPATCHERS
    }

    private fun isProvider(expression: KtNameReferenceExpression): Boolean {
        val property = expression.getStrictParentOfType<KtPropertyAccessor>()?.property
            ?: expression.getStrictParentOfType<KtProperty>()
        return property?.hasModifier(KtTokens.OVERRIDE_KEYWORD) == true
    }

    private fun isInDi(expression: KtNameReferenceExpression): Boolean {
        val segments = expression.containingKtFile.packageFqName.pathSegments().map(Name::asString)
        return DI_SEGMENT in segments
    }

    override fun visitReferenceExpression(expression: KtReferenceExpression) {
        super.visitReferenceExpression(expression)
        val reference = expression as? KtNameReferenceExpression ?: return
        val name = reference.getReferencedName()
        if (name !in dispatcherNames || reference.getStrictParentOfType<KtImportDirective>() != null) return
        if (reference.containingKtFile.isTestSource() || isInDi(reference) || isProvider(reference)) return
        if (!isKotlinxDispatcher(reference)) return
        val message = "Take the $name dispatcher from the injected dispatchers instead of Dispatchers.$name: a class " +
            "gets its dispatchers through the constructor, so a test can replace them."
        report(Finding(Entity.from(reference), message))
    }

    private companion object {
        const val COROUTINES_PACKAGE = "kotlinx.coroutines"
        const val DISPATCHERS = "Dispatchers"
        const val DI_SEGMENT = "di"
        const val DESCRIPTION = "A class takes its dispatchers through the constructor, never from Dispatchers."
    }
}
