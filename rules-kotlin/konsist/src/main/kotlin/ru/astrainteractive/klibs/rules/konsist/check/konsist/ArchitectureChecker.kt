package ru.astrainteractive.klibs.rules.konsist.check.konsist

import ru.astrainteractive.klibs.rules.konsist.check.model.Violation

/**
 * Runs every check whose unit kinds include the kind of the unit, and orders the violations by file.
 */
internal class ArchitectureChecker(
    private val checks: List<ArchitectureCheck>,
    private val suppressionPolicy: SuppressionPolicy
) {
    fun check(sources: UnitSources): List<Violation> {
        return checks
            .filter { check -> sources.kind in check.unitKinds }
            .flatMap { check -> check.check(sources) }
            .filterNot(suppressionPolicy::isSuppressed)
            .sortedWith(
                compareBy(
                    Violation::path,
                    Violation::rule,
                    Violation::subject
                )
            )
    }
}
