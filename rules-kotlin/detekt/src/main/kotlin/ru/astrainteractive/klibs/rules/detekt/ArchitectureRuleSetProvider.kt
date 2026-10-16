package ru.astrainteractive.klibs.rules.detekt

import dev.detekt.api.RuleSet
import dev.detekt.api.RuleSetId
import dev.detekt.api.RuleSetProvider
import ru.astrainteractive.klibs.rules.detekt.declaration.DataClassDefaultValue
import ru.astrainteractive.klibs.rules.detekt.declaration.LocalOrInnerClass
import ru.astrainteractive.klibs.rules.detekt.injection.ConstructorDefaultDependency
import ru.astrainteractive.klibs.rules.detekt.kdoc.KDocHtmlTag
import ru.astrainteractive.klibs.rules.detekt.lambda.ImplicitItParameter
import ru.astrainteractive.klibs.rules.detekt.naming.ForbiddenPackageSegment
import ru.astrainteractive.klibs.rules.detekt.naming.ImplementationName
import ru.astrainteractive.klibs.rules.detekt.naming.KindNameInPackage
import ru.astrainteractive.klibs.rules.detekt.ordering.CalleeBeforeCaller
import ru.astrainteractive.klibs.rules.detekt.tuple.AnonymousTuple

/**
 * Registers the architecture rules with detekt. A rule runs only when `config/config.yml` lists it.
 */
public class ArchitectureRuleSetProvider : RuleSetProvider {
    override val ruleSetId: RuleSetId = RuleSetId(RULE_SET_ID)

    override fun instance(): RuleSet = RuleSet(
        ruleSetId,
        listOf(
            ::AnonymousTuple,
            ::CalleeBeforeCaller,
            ::ConstructorDefaultDependency,
            ::DataClassDefaultValue,
            ::ForbiddenPackageSegment,
            ::ImplementationName,
            ::ImplicitItParameter,
            ::KDocHtmlTag,
            ::KindNameInPackage,
            ::LocalOrInnerClass,
        )
    )

    internal companion object {
        const val RULE_SET_ID = "architecture"
    }
}
