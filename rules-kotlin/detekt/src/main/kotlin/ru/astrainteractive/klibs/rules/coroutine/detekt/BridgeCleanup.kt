package ru.astrainteractive.klibs.rules.coroutine.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.RequiresAnalysisApi
import dev.detekt.api.Rule
import org.jetbrains.kotlin.analysis.api.analyze
import org.jetbrains.kotlin.analysis.api.resolution.KaExplicitReceiverValue
import org.jetbrains.kotlin.analysis.api.resolution.KaImplicitReceiverValue
import org.jetbrains.kotlin.analysis.api.resolution.KaReceiverValue
import org.jetbrains.kotlin.analysis.api.resolution.KaSmartCastedReceiverValue
import org.jetbrains.kotlin.analysis.api.resolution.singleFunctionCallOrNull
import org.jetbrains.kotlin.analysis.api.resolution.symbol
import org.jetbrains.kotlin.idea.references.mainReference
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtArrayAccessExpression
import org.jetbrains.kotlin.psi.KtBinaryExpression
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtLambdaExpression
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtParameter
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtThisExpression
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType
import org.jetbrains.kotlin.psi.psiUtil.getStrictParentOfType
import org.jetbrains.kotlin.psi.psiUtil.parentsWithSelf

/**
 * Reports a bridge cleanup out of shape: the lambda of `awaitClose` or `invokeOnCancellation` that assigns state
 * declared outside its bridge, or launches on the bridge's cancelled scope. References are resolved before
 * ownership is judged: a registration-local object is safe, an alias of external state is not local ownership,
 * and an independent cleanup scope is not assumed to be cancelled. Unknown factories remain a review decision.
 */
internal class BridgeCleanup(config: Config) : Rule(config, DESCRIPTION), RequiresAnalysisApi {

    private fun functionId(call: KtCallExpression): String? {
        return analyze(call) {
            val symbol = call.resolveToCall()?.singleFunctionCallOrNull()?.symbol
            symbol?.callableId?.asSingleFqName()?.asString()
        }
    }

    private fun rootReference(expression: KtExpression): KtNameReferenceExpression? {
        return when (expression) {
            is KtNameReferenceExpression -> expression
            is KtDotQualifiedExpression -> if (expression.receiverExpression is KtThisExpression) {
                expression.selectorExpression as? KtNameReferenceExpression
            } else {
                rootReference(expression.receiverExpression)
            }
            is KtArrayAccessExpression -> expression.arrayExpression?.let(::rootReference)
            else -> null
        }
    }

    private fun isOutside(
        expression: KtExpression,
        bridge: KtLambdaExpression,
        followAliases: Boolean,
        seen: Set<KtProperty>
    ): Boolean {
        val reference = rootReference(expression) ?: return false
        val declaration = analyze(reference) { reference.mainReference.resolveToSymbol()?.psi }
        if (declaration is KtParameter) return !bridge.textRange.contains(declaration.textRange)
        if (declaration !is KtProperty || declaration in seen) return false
        if (!bridge.textRange.contains(declaration.textRange)) return true
        if (!followAliases) return false
        val value = declaration.initializer ?: return false
        return isOutside(
            value,
            bridge,
            true,
            seen.plus(declaration)
        )
    }

    private fun assignedOutside(bridge: KtLambdaExpression, cleanup: KtLambdaExpression): List<KtBinaryExpression> {
        return cleanup.collectDescendantsOfType<KtBinaryExpression> { expression ->
            expression.operationToken in KtTokens.ALL_ASSIGNMENTS
        }.filter { assignment ->
            val left = assignment.left ?: return@filter false
            isOutside(
                left,
                bridge,
                left !is KtNameReferenceExpression,
                emptySet()
            )
        }
    }

    private fun isBridgeReference(
        expression: KtExpression,
        bridge: KtLambdaExpression,
        seen: Set<KtProperty>
    ): Boolean {
        if (expression is KtThisExpression) {
            return analyze(expression) {
                val symbol = expression.instanceReference.mainReference.resolveToSymbol()
                symbol?.psi == bridge.functionLiteral || symbol?.containingSymbol?.psi == bridge.functionLiteral
            }
        }
        val reference = expression as? KtNameReferenceExpression ?: return false
        val declaration = analyze(reference) { reference.mainReference.resolveToSymbol()?.psi }
        val property = declaration as? KtProperty ?: return false
        if (property in seen) return false
        val initializer = property.initializer ?: return false
        return isBridgeReference(
            initializer,
            bridge,
            seen.plus(property)
        )
    }

    private fun isBridgeReceiver(receiver: KaReceiverValue, bridge: KtLambdaExpression): Boolean {
        return when (receiver) {
            is KaImplicitReceiverValue -> receiver.symbol.psi == bridge.functionLiteral
            is KaExplicitReceiverValue -> isBridgeReference(
                receiver.expression,
                bridge,
                emptySet()
            )
            is KaSmartCastedReceiverValue -> isBridgeReceiver(receiver.original, bridge)
        }
    }

    private fun launchesOnBridge(call: KtCallExpression, bridge: KtLambdaExpression): Boolean {
        return analyze(call) {
            val resolved = call.resolveToCall()?.singleFunctionCallOrNull() ?: return@analyze false
            val id = resolved.symbol.callableId?.asSingleFqName()?.asString()
            if (id != "kotlinx.coroutines.launch") return@analyze false
            val receiver = resolved.extensionReceiver ?: return@analyze false
            isBridgeReceiver(receiver, bridge)
        }
    }

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        val id = functionId(expression)
        if (id !in CLEANUP_CALLS) return
        val name = id?.substringAfterLast('.')
        val lambda = expression.lambdaArguments.firstOrNull()?.getLambdaExpression() ?: return
        val bridge = expression.parentsWithSelf.filterIsInstance<KtLambdaExpression>().firstOrNull { candidate ->
            val call = candidate.getStrictParentOfType<KtCallExpression>() ?: return@firstOrNull false
            functionId(call) in BRIDGE_CALLS
        } ?: return
        assignedOutside(bridge, lambda).forEach { assignment ->
            val assigned = assignment.left?.text
            val message = "Keep the $name cleanup to what this bridge registered: it assigns '$assigned' declared " +
                "outside it, which races with the next registration."
            report(Finding(Entity.from(assignment), message))
        }
        lambda.collectDescendantsOfType<KtCallExpression> { call -> launchesOnBridge(call, bridge) }
            .forEach { launch ->
                val message = "Unregister synchronously in $name: a launch there runs on the cancelled scope and " +
                    "dies silently; a suspending unregister runs on a scope the class owns."
                report(Finding(Entity.from(launch), message))
            }
    }

    private companion object {
        @JvmField
        val CLEANUP_CALLS = setOf(
            "kotlinx.coroutines.channels.awaitClose",
            "kotlinx.coroutines.CancellableContinuation.invokeOnCancellation"
        )

        @JvmField
        val BRIDGE_CALLS = setOf(
            "kotlinx.coroutines.flow.callbackFlow",
            "kotlinx.coroutines.flow.channelFlow",
            "kotlinx.coroutines.suspendCancellableCoroutine"
        )
        const val DESCRIPTION = "The cleanup of a bridge removes only what it registered, synchronously."
    }
}
