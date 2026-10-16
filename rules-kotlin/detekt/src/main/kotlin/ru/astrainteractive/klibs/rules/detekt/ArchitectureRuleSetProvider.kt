package ru.astrainteractive.klibs.rules.detekt

import dev.detekt.api.RuleSet
import dev.detekt.api.RuleSetId
import dev.detekt.api.RuleSetProvider
import ru.astrainteractive.klibs.rules.detekt.lambda.ImplicitItParameter

/**
 * Registers the architecture rules with detekt. A rule runs only when `config/config.yml` lists it.
 */
public class ArchitectureRuleSetProvider : RuleSetProvider {
    override val ruleSetId: RuleSetId = RuleSetId(RULE_SET_ID)

    override fun instance(): RuleSet = RuleSet(
        ruleSetId,
        listOf(
            ::ImplicitItParameter,
        )
    )

    internal companion object {
        const val RULE_SET_ID = "architecture"
    }
}
