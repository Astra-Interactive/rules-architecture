package ru.astrainteractive.klibs.rules.detekt

import dev.detekt.api.Config
import dev.detekt.api.Rule
import dev.detekt.api.RuleSet
import dev.detekt.api.RuleSetId
import dev.detekt.api.RuleSetProvider
import ru.astrainteractive.klibs.rules.compose.detekt.ComposableFileType
import ru.astrainteractive.klibs.rules.compose.detekt.ComposableWhenBranch
import ru.astrainteractive.klibs.rules.compose.detekt.ComposeConstantCase
import ru.astrainteractive.klibs.rules.compose.detekt.NonComposableFileInComposable
import ru.astrainteractive.klibs.rules.compose.detekt.PreviewBlockLast
import ru.astrainteractive.klibs.rules.compose.detekt.RememberInName
import ru.astrainteractive.klibs.rules.compose.detekt.ThemeObjectFunction
import ru.astrainteractive.klibs.rules.coroutine.detekt.CallbackFlowShape
import ru.astrainteractive.klibs.rules.coroutine.detekt.ImpureUpdateLambda
import ru.astrainteractive.klibs.rules.coroutine.detekt.MutableStateFlowEmit
import ru.astrainteractive.klibs.rules.coroutine.detekt.RunBlockingOutsideBoundary
import ru.astrainteractive.klibs.rules.coroutine.detekt.SingleJobLaunchMode
import ru.astrainteractive.klibs.rules.coroutine.detekt.StateFlowTimeout
import ru.astrainteractive.klibs.rules.coroutine.detekt.ThreadPrimitive
import ru.astrainteractive.klibs.rules.declaration.detekt.BareIdentifier
import ru.astrainteractive.klibs.rules.declaration.detekt.CallbackDeclaration
import ru.astrainteractive.klibs.rules.declaration.detekt.CompanionFactoryFunction
import ru.astrainteractive.klibs.rules.declaration.detekt.DataClassDefaultValue
import ru.astrainteractive.klibs.rules.declaration.detekt.LocalOrInnerClass
import ru.astrainteractive.klibs.rules.declaration.detekt.NamespaceObject
import ru.astrainteractive.klibs.rules.declaration.detekt.OneTypeFamilyPerFile
import ru.astrainteractive.klibs.rules.declaration.detekt.PropertyAsFunction
import ru.astrainteractive.klibs.rules.formatting.detekt.ArgumentPerLine
import ru.astrainteractive.klibs.rules.formatting.detekt.BooleanOperatorWrapping
import ru.astrainteractive.klibs.rules.formatting.detekt.ExplicitInvoke
import ru.astrainteractive.klibs.rules.formatting.detekt.ExpressionBodyOnOneLine
import ru.astrainteractive.klibs.rules.formatting.detekt.OperatorBeforeTrailingCall
import ru.astrainteractive.klibs.rules.injection.detekt.ConstructorDefaultDependency
import ru.astrainteractive.klibs.rules.injection.detekt.DiModuleLifecycle
import ru.astrainteractive.klibs.rules.injection.detekt.RootModuleConstruction
import ru.astrainteractive.klibs.rules.kdoc.detekt.KDocFiller
import ru.astrainteractive.klibs.rules.kdoc.detekt.KDocHtmlTag
import ru.astrainteractive.klibs.rules.lambda.detekt.ImplicitItParameter
import ru.astrainteractive.klibs.rules.lambda.detekt.LambdaSingleIfGuard
import ru.astrainteractive.klibs.rules.lambda.detekt.TrivialLambdaReference
import ru.astrainteractive.klibs.rules.multiplatform.detekt.ActualFileSuffix
import ru.astrainteractive.klibs.rules.naming.detekt.FunctionVerb
import ru.astrainteractive.klibs.rules.naming.detekt.ImplementationName
import ru.astrainteractive.klibs.rules.naming.detekt.KindNameInPackage
import ru.astrainteractive.klibs.rules.naming.detekt.RolelessClassName
import ru.astrainteractive.klibs.rules.naming.detekt.TestFunctionName
import ru.astrainteractive.klibs.rules.ordering.detekt.CalleeBeforeCaller
import ru.astrainteractive.klibs.rules.ordering.detekt.ConstantOutsideCompanion
import ru.astrainteractive.klibs.rules.ordering.detekt.DeclarationOrder
import ru.astrainteractive.klibs.rules.result.detekt.GetOrThrowInMain
import ru.astrainteractive.klibs.rules.result.detekt.ResultConsumption
import ru.astrainteractive.klibs.rules.result.detekt.RunCatchingCancellation
import ru.astrainteractive.klibs.rules.result.detekt.ThrowInsideRunCatching
import ru.astrainteractive.klibs.rules.testing.detekt.RunBlockingInTest
import ru.astrainteractive.klibs.rules.time.detekt.DurationOverloadCall
import ru.astrainteractive.klibs.rules.time.detekt.SystemTimeCall
import ru.astrainteractive.klibs.rules.time.detekt.TimeAsPrimitive
import ru.astrainteractive.klibs.rules.tuple.detekt.AnonymousTuple

/**
 * Registers the architecture rules with detekt. A rule runs only when `config/config.yml` lists it.
 */
public class ArchitectureRuleSetProvider : RuleSetProvider {
    override val ruleSetId: RuleSetId = RuleSetId(RULE_SET_ID)

    override fun instance(): RuleSet {
        return RuleSet(
            ruleSetId,
            RULES
        )
    }

    internal companion object {
        const val RULE_SET_ID = "architecture"

        private val RULES: List<(Config) -> Rule> = listOf(
            ::ActualFileSuffix,
            ::AnonymousTuple,
            ::ArgumentPerLine,
            ::BareIdentifier,
            ::BooleanOperatorWrapping,
            ::CallbackDeclaration,
            ::CallbackFlowShape,
            ::CalleeBeforeCaller,
            ::CompanionFactoryFunction,
            ::ComposableFileType,
            ::ComposableWhenBranch,
            ::ComposeConstantCase,
            ::ConstantOutsideCompanion,
            ::ConstructorDefaultDependency,
            ::DataClassDefaultValue,
            ::DeclarationOrder,
            ::DiModuleLifecycle,
            ::DurationOverloadCall,
            ::ExplicitInvoke,
            ::ExpressionBodyOnOneLine,
            ::FunctionVerb,
            ::GetOrThrowInMain,
            ::ImplementationName,
            ::ImplicitItParameter,
            ::ImpureUpdateLambda,
            ::KDocFiller,
            ::KDocHtmlTag,
            ::KindNameInPackage,
            ::LambdaSingleIfGuard,
            ::LocalOrInnerClass,
            ::MutableStateFlowEmit,
            ::NamespaceObject,
            ::NonComposableFileInComposable,
            ::OneTypeFamilyPerFile,
            ::OperatorBeforeTrailingCall,
            ::PreviewBlockLast,
            ::PropertyAsFunction,
            ::RememberInName,
            ::ResultConsumption,
            ::RolelessClassName,
            ::RootModuleConstruction,
            ::RunBlockingInTest,
            ::RunBlockingOutsideBoundary,
            ::RunCatchingCancellation,
            ::SingleJobLaunchMode,
            ::StateFlowTimeout,
            ::SystemTimeCall,
            ::TestFunctionName,
            ::ThemeObjectFunction,
            ::ThreadPrimitive,
            ::ThrowInsideRunCatching,
            ::TimeAsPrimitive,
            ::TrivialLambdaReference,
        )
    }
}
