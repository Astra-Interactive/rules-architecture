package ru.astrainteractive.klibs.rules.konsist.check.placement

import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.Layer
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a non-private error type outside `model` and outside a technology package. An error is recognized by
 * a supertype whose name ends with `Exception`, `Error` or `Throwable`.
 */
internal class ErrorPlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = setOf(UnitKind.CONTEXT)

    override fun check(sources: UnitSources): List<Violation> = sources.mainFiles
        .filter { file -> sources.placementOf(file)?.layer.let { layer -> layer != null && layer !in ALLOWED_LAYERS } }
        .flatMap { file ->
            file.topLevelClasses()
                .filterNot { declaration -> declaration.hasPrivateModifier }
                .filter { declaration -> declaration.parents().any { parent -> ERROR_SUPERTYPE.matches(parent.name) } }
                .map { declaration ->
                    val message = "Move ${declaration.name} to model: an error family lives in model, next to the " +
                        "types it concerns."
                    Violation(RULE, sources.relativePath(file), declaration.name, message)
                }
        }

    private companion object {
        const val RULE = "ErrorPlacement"

        @JvmField
        val ALLOWED_LAYERS = setOf(Layer.MODEL, Layer.TECHNOLOGY)

        @JvmField
        val ERROR_SUPERTYPE = Regex("\\w*(Exception|Error|Throwable)")
    }
}
