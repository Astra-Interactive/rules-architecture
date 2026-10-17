package ru.astrainteractive.klibs.rules.konsist.check.placement

import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClassAndObjectNames
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a DI module, a class or object named `*Module`, outside a `di` package of the level it wires.
 */
internal class DiPlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles
            .filter { file -> sources.placementOf(file)?.kind != Kind.DI }
            .flatMap { file ->
                file.topLevelClassAndObjectNames()
                    .filter { name -> name.endsWith(MODULE_SUFFIX) }
                    .map { name ->
                        val message = "Move $name to the di package of the level it wires."
                        Violation(RULE, sources.relativePath(file), name, message)
                    }
            }
    }

    private companion object {
        const val RULE = "DiPlacement"
        const val MODULE_SUFFIX = "Module"
    }
}
