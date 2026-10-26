package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.simpleName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelTypeNames
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a top-level function in `util` that takes exactly one parameter whose type is a model of the unit: a
 * rule about one type is an extension of that type.
 */
internal class UtilFunctionAsExtension : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    override fun check(sources: UnitSources): List<Violation> {
        val modelNames = sources.mainFiles
            .filter { file -> sources.placementOf(file)?.kind == Kind.MODEL }
            .flatMap(KoFileDeclaration::topLevelTypeNames)
            .toSet()
        return sources.mainFiles
            .filter { file -> sources.placementOf(file)?.kind == Kind.UTIL }
            .flatMap { file ->
                file.functions(includeNested = false, includeLocal = false)
                    .filter { function -> function.receiverType == null }
                    .mapNotNull { function ->
                        val typeName = function.parameters.singleOrNull()?.type?.simpleName ?: return@mapNotNull null
                        if (typeName !in modelNames) return@mapNotNull null
                        val message = "Write ${function.name} as an extension of $typeName: " +
                            "fun $typeName.${function.name}(), a rule about one type extends that type."
                        sources.createViolation(
                            RULE,
                            file,
                            function,
                            function.name,
                            message
                        )
                    }
            }
    }

    private companion object {
        const val RULE = "UtilFunctionAsExtension"
    }
}
