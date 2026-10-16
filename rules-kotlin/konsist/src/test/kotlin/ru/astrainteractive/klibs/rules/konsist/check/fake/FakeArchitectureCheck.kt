package ru.astrainteractive.klibs.rules.konsist.check.fake

import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Returns the scripted violations and records how many times it ran.
 */
internal class FakeArchitectureCheck(
    override val unitKinds: Set<UnitKind>,
    private val violations: List<Violation>
) : ArchitectureCheck {
    var runs: Int = 0
        private set

    override fun check(sources: UnitSources): List<Violation> {
        runs++
        return violations
    }
}
