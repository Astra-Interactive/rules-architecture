package ru.astrainteractive.klibs.rules.konsist.check.konsist

import ru.astrainteractive.klibs.rules.core.unit.model.RuleId
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation

/**
 * Runs every check whose unit kinds include the kind of the unit, and orders the violations by file.
 */
internal class ArchitectureChecker(
    private val checks: Map<RuleId, ArchitectureCheck>,
    private val suppressionPolicy: SuppressionPolicy
) {
    fun check(sources: UnitSources): List<Violation> {
        val disabledRules = sources.context.policy.checks.disabledRules.keys
        return checks
            .filter { (ruleId, check) ->
                sources.kind in check.unitKinds && ruleId.ruleName !in disabledRules
            }
            .flatMap { (_, check) -> check.check(sources) }
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
