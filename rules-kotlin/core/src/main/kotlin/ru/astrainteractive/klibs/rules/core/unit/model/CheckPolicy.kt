package ru.astrainteractive.klibs.rules.core.unit.model

/** Normalized, serializable configuration shared by all runners. */
public data class CheckPolicy(
    public val disabledRules: Map<String, String>,
    public val allowedDependencies: Map<String, String>
) {
    init {
        disabledRules.forEach { (name, reason) ->
            require(RuleId.entries.any { rule -> rule.ruleName == name }) { "Unknown rule '$name'" }
            require(reason.isNotBlank()) { "Disabled rule '$name' requires a reason" }
        }
        allowedDependencies.forEach { (path, reason) ->
            UnitReference(path)
            require(reason.isNotBlank()) { "Dependency allowance '$path' requires a reason" }
        }
    }
}
