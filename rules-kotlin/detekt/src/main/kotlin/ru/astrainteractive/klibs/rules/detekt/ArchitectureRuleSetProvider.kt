package ru.astrainteractive.klibs.rules.detekt

import dev.detekt.api.RuleSet
import dev.detekt.api.RuleSetId
import dev.detekt.api.RuleSetProvider
import ru.astrainteractive.klibs.rules.detekt.compose.ComposableFileType
import ru.astrainteractive.klibs.rules.detekt.compose.ComposeConstantCase
import ru.astrainteractive.klibs.rules.detekt.compose.ThemeObjectFunction
import ru.astrainteractive.klibs.rules.detekt.declaration.DataClassDefaultValue
import ru.astrainteractive.klibs.rules.detekt.declaration.LocalOrInnerClass
import ru.astrainteractive.klibs.rules.detekt.declaration.NamespaceObject
import ru.astrainteractive.klibs.rules.detekt.declaration.OneTypeFamilyPerFile
import ru.astrainteractive.klibs.rules.detekt.declaration.PropertyAsFunction
import ru.astrainteractive.klibs.rules.detekt.formatting.ArgumentPerLine
import ru.astrainteractive.klibs.rules.detekt.formatting.BooleanOperatorWrapping
import ru.astrainteractive.klibs.rules.detekt.formatting.ExplicitInvoke
import ru.astrainteractive.klibs.rules.detekt.formatting.ExpressionBodyOnOneLine
import ru.astrainteractive.klibs.rules.detekt.injection.ConstructorDefaultDependency
import ru.astrainteractive.klibs.rules.detekt.kdoc.KDocHtmlTag
import ru.astrainteractive.klibs.rules.detekt.lambda.ImplicitItParameter
import ru.astrainteractive.klibs.rules.detekt.naming.FunctionVerb
import ru.astrainteractive.klibs.rules.detekt.naming.ImplementationName
import ru.astrainteractive.klibs.rules.detekt.naming.KindNameInPackage
import ru.astrainteractive.klibs.rules.detekt.naming.TestFunctionName
import ru.astrainteractive.klibs.rules.detekt.ordering.CalleeBeforeCaller
import ru.astrainteractive.klibs.rules.detekt.ordering.ConstantOutsideCompanion
import ru.astrainteractive.klibs.rules.detekt.ordering.DeclarationOrder
import ru.astrainteractive.klibs.rules.detekt.time.TimeAsPrimitive
import ru.astrainteractive.klibs.rules.detekt.tuple.AnonymousTuple

/**
 * Registers the architecture rules with detekt. A rule runs only when `config/config.yml` lists it.
 */
public class ArchitectureRuleSetProvider : RuleSetProvider {
    override val ruleSetId: RuleSetId = RuleSetId(RULE_SET_ID)

    override fun instance(): RuleSet {
        return RuleSet(
            ruleSetId,
            listOf(
                ::AnonymousTuple,
                ::ArgumentPerLine,
                ::BooleanOperatorWrapping,
                ::CalleeBeforeCaller,
                ::ComposableFileType,
                ::ComposeConstantCase,
                ::ConstantOutsideCompanion,
                ::ConstructorDefaultDependency,
                ::DataClassDefaultValue,
                ::DeclarationOrder,
                ::ExplicitInvoke,
                ::ExpressionBodyOnOneLine,
                ::FunctionVerb,
                ::ImplementationName,
                ::ImplicitItParameter,
                ::KDocHtmlTag,
                ::KindNameInPackage,
                ::LocalOrInnerClass,
                ::NamespaceObject,
                ::OneTypeFamilyPerFile,
                ::PropertyAsFunction,
                ::TestFunctionName,
                ::ThemeObjectFunction,
                ::TimeAsPrimitive,
            )
        )
    }

    internal companion object {
        const val RULE_SET_ID = "architecture"
    }
}
