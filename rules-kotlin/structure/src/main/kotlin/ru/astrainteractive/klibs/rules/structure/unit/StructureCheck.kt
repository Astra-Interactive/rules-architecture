package ru.astrainteractive.klibs.rules.structure.unit

/**
 * One structure rule checked over a build unit.
 */
public fun interface StructureCheck {
    public fun check(unit: BuildUnit): List<StructureViolation>
}
