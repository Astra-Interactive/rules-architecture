package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a DI module, a class or object named `*Module`, outside a `di` package of the level it wires.
 */
internal class DiPlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles
            .filter { file -> sources.placementOf(file)?.kind != Kind.DI }
            .flatMap { file ->
                file.topLevelClasses()
                    .plus(file.objects(includeNested = false))
                    .filter { declaration -> declaration.name.endsWith(MODULE_SUFFIX) }
                    .map { declaration ->
                        val message = "Move ${declaration.name} to the di package of the level it wires."
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
        const val RULE = "DiPlacement"
        const val MODULE_SUFFIX = "Module"
    }
}
