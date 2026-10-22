package ru.astrainteractive.klibs.rules.konsist.check.placement

import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.simpleName
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelTypeNames
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a top-level function in `util` that takes exactly one parameter whose type is a model of the unit: a
 * rule about one type is an extension of that type.
 */
internal class UtilFunctionAsExtension : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    override fun check(sources: UnitSources): List<Violation> {
        val modelNames = sources.mainFiles
            .filter { file -> sources.placementOf(file)?.kind == Kind.MODEL }
            .flatMap { file -> file.topLevelTypeNames() }
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
                        Violation(
                            RULE,
                            sources.relativePath(file),
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
