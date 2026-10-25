package ru.astrainteractive.klibs.rules.structure.unit.structure

import ru.astrainteractive.klibs.rules.structure.unit.api.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation

/**
 * Runs every structure check over a build unit.
 */
public class StructureChecker(private val checks: List<StructureCheck>) {
    public fun check(unit: BuildUnit): List<StructureViolation> {
        return checks
            .flatMap { check -> check.check(unit) }
            .sortedWith(
                compareBy(
                    StructureViolation::path,
                    StructureViolation::rule,
                    StructureViolation::subject
                )
            )
    }
}
