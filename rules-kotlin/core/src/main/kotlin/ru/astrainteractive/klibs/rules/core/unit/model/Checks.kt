package ru.astrainteractive.klibs.rules.core.unit.model

/** Explicit exceptions; the reason is included in the lint policy report. */
public class Checks(
    disabled: Map<RuleId, String> = emptyMap(),
    allowedDependencies: Map<UnitReference, String> = emptyMap()
) {
    public val disabled: Map<RuleId, String> = disabled.toMap()
    public val allowedDependencies: Map<UnitReference, String> = allowedDependencies.toMap()

    init {
        require(this.disabled.values.all(String::isNotBlank)) { "Disabling a rule requires a reason" }
        require(this.allowedDependencies.values.all(String::isNotBlank)) {
            "Allowing a dependency requires a reason"
        }
    }
}
