package ru.astrainteractive.klibs.rules.konsist.check.konsist

import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

// Kept beside the checks it types by the owner's decision of 2026-10-31.

/**
 * One architecture rule checked over the sources of a build unit.
 */
@Suppress("ContractPlacement")
internal interface ArchitectureCheck {
    /**
     * The unit kinds whose layout this rule describes; the check is skipped for any other kind.
     */
    val unitKinds: Set<UnitKind>

    fun check(sources: UnitSources): List<Violation>
}
