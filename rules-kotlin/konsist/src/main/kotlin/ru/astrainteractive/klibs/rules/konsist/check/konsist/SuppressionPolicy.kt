package ru.astrainteractive.klibs.rules.konsist.check.konsist

import com.lemonappdev.konsist.api.declaration.KoAnnotationDeclaration
import com.lemonappdev.konsist.api.declaration.KoArgumentDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.provider.KoAnnotationProvider
import com.lemonappdev.konsist.api.provider.KoContainingDeclarationProvider
import ru.astrainteractive.klibs.rules.konsist.check.model.DeclarationOrigin
import ru.astrainteractive.klibs.rules.konsist.check.model.ExternalOrigin
import ru.astrainteractive.klibs.rules.konsist.check.model.FileOrigin
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation

private const val SUPPRESS = "Suppress"

internal class SuppressionPolicy {
    private fun suppressedRules(declaration: KoAnnotationProvider): Set<String> {
        return declaration.annotations
            .filter { annotation -> annotation.name == SUPPRESS }
            .flatMap(KoAnnotationDeclaration::arguments)
            .mapNotNull(KoArgumentDeclaration::value)
            .toSet()
    }

    fun isSuppressed(violation: Violation): Boolean {
        var declaration = when (val origin = violation.origin) {
            is DeclarationOrigin -> origin.declaration
            is FileOrigin -> origin.file
            is ExternalOrigin -> return false
        }
        while (true) {
            val annotatedDeclaration = declaration as? KoAnnotationProvider
            val rules = annotatedDeclaration?.let(::suppressedRules).orEmpty()
            if (violation.rule in rules) return true
            if (declaration is KoFileDeclaration) return false
            declaration = (declaration as? KoContainingDeclarationProvider)?.containingDeclaration ?: return false
        }
    }
}
