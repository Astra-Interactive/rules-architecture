package ru.astrainteractive.klibs.rules.gradle.linter.util

import ru.astrainteractive.klibs.rules.core.unit.model.RuleId
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext

internal fun UnitContext.withDisabledRules(rules: Collection<RuleId>, reason: String): UnitContext {
    val disabledRules = buildMap {
        rules.forEach { rule -> put(rule.ruleName, reason) }
        putAll(policy.checks.disabledRules)
    }
    val checks = policy.checks.copy(disabledRules = disabledRules.toSortedMap())
    return copy(policy = policy.copy(checks = checks))
}
