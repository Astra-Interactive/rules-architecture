package ru.astrainteractive.klibs.rules.structure.unit.api

import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation

/**
 * One structure rule checked over a build unit.
 */
public fun interface StructureCheck {
    public fun check(unit: BuildUnit): List<StructureViolation>
}
