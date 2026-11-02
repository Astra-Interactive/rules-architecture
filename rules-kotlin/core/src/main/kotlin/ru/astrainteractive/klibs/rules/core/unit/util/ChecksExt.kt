package ru.astrainteractive.klibs.rules.core.unit.util

import ru.astrainteractive.klibs.rules.core.unit.model.CheckPolicy
import ru.astrainteractive.klibs.rules.core.unit.model.Checks

public fun Checks.toPolicy(): CheckPolicy {
    return CheckPolicy(
        disabled.mapKeys { (rule, _) -> rule.ruleName }.toSortedMap(),
        allowedDependencies.mapKeys { (reference, _) -> reference.path }.toSortedMap()
    )
}
