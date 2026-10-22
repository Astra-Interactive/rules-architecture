package ru.astrainteractive.klibs.rules.structure.unit

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
