package ru.astrainteractive.klibs.rules.structure.graph.structure

import ru.astrainteractive.klibs.rules.structure.unit.api.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation

/**
 * Checks the unit table of `arch-context.md` from one unit's own dependencies, reading unit kinds from the
 * project path: a `contract` unit depends on nothing of the project, `core` on no feature unit, and nothing but
 * the entry point depends on an `impl` unit or on a feature's platform unit (`gui/bukkit`). `core`'s own
 * platform unit (`core/bukkit`) is part of `core`.
 */
public class UnitDependency : StructureCheck {
    override fun check(unit: BuildUnit): List<StructureViolation> {
        return UnitDependencyDirection.findForbidden(unit).map { dependency ->
            StructureViolation(
                RULE,
                BUILD_FILE,
                dependency,
                UnitDependencyDirection.message(unit, dependency)
            )
        }
    }

    private companion object {
        const val RULE = "UnitDependency"
        const val BUILD_FILE = "build.gradle.kts"
    }
}
