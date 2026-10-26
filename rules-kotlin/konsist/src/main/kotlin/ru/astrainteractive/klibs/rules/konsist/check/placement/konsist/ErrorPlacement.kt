package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports an error type outside a `model` package: every error type is its own file in the `model` of the
 * level that uses it, even one only one class throws. An error is recognized by a supertype whose name ends
 * with `Exception`, `Error` or `Throwable`.
 */
internal class ErrorPlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun isOutsideModel(sources: UnitSources, file: KoFileDeclaration): Boolean {
        val placement = sources.placementOf(file) ?: return false
        return placement.kind != Kind.MODEL
    }

    private fun isError(declaration: KoClassDeclaration): Boolean {
        return declaration.parents().any { parent -> ERROR_SUPERTYPE.matches(parent.name) }
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles
            .filter { file -> isOutsideModel(sources, file) }
            .flatMap { file ->
                file.topLevelClasses()
                    .filter(::isError)
                    .map { declaration ->
                        val message = "Move ${declaration.name} to the model package of the level that uses it: " +
                            "every error type is its own file in model."
                        sources.createViolation(
                            RULE,
                            file,
                            declaration,
                            declaration.name,
                            message
                        )
                    }
            }
    }

    private companion object {
        const val RULE = "ErrorPlacement"

        @JvmField
        val ERROR_SUPERTYPE = Regex("""\w*(Exception|Error|Throwable)""")
    }
}
