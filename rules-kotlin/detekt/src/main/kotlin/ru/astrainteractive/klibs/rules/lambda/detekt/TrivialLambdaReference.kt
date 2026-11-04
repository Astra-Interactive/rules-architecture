package ru.astrainteractive.klibs.rules.lambda.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.RequiresAnalysisApi
import dev.detekt.api.Rule
import org.jetbrains.kotlin.analysis.api.analyze
import org.jetbrains.kotlin.analysis.api.resolution.singleFunctionCallOrNull
import org.jetbrains.kotlin.analysis.api.resolution.symbol
import org.jetbrains.kotlin.analysis.api.symbols.KaKotlinPropertySymbol
import org.jetbrains.kotlin.analysis.api.symbols.KaLocalVariableSymbol
import org.jetbrains.kotlin.analysis.api.symbols.KaNamedClassSymbol
import org.jetbrains.kotlin.analysis.api.symbols.KaNamedFunctionSymbol
import org.jetbrains.kotlin.analysis.api.symbols.KaSymbolModality
import org.jetbrains.kotlin.analysis.api.symbols.KaSyntheticJavaPropertySymbol
import org.jetbrains.kotlin.analysis.api.symbols.KaValueParameterSymbol
import org.jetbrains.kotlin.analysis.api.symbols.KaVariableSymbol
import org.jetbrains.kotlin.analysis.api.types.KaFunctionType
import org.jetbrains.kotlin.idea.references.mainReference
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtDotQualifiedExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtFile
import org.jetbrains.kotlin.psi.KtLambdaExpression
import org.jetbrains.kotlin.psi.KtNameReferenceExpression
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtParameter
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.KtValueArgument
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType
import ru.astrainteractive.klibs.rules.detekt.util.isSuspend

/**
 * Reports a lambda whose whole body is one call that receives the lambda's parameters unchanged and in order
 * (`{ p -> foo(p) }`, `{ p -> Foo(p) }`, `{ p -> p.foo() }`, `{ p -> p.prop }`, `{ p -> obj.foo(p) }`), and a
 * callback `{ obj.foo() }` passed as a named argument or stored in a property: a function reference says the same.
 *
 * It stays silent where a reference may not compile or may mean another function: for a resolved `suspend`
 * callee unless the receiving function type is also `suspend`, for a callee declared in the file more than once,
 * and for type, named, spread or trailing-lambda
 * arguments, for an extension declared as a member of a class, which no reference can name, and for a Java getter
 * read as a property (`record.level`), whose reference does not compile, and for a property read in a `suspend`
 * lambda (`flow.map { message -> message.value }`), since a property reference is no `suspend` function. A
 * companion function of `Result` is suggested as an imported `::success`, since `Result::success` does not compile.
 * Bound references require a stable receiver: capturing a mutable or computed receiver would change when it is read.
 */
internal class TrivialLambdaReference(config: Config) : Rule(config, DESCRIPTION), RequiresAnalysisApi {
    private var functionCounts: Map<String, Int> = emptyMap()

    private fun isCallback(lambda: KtLambdaExpression): Boolean {
        val parent = lambda.parent
        return parent is KtProperty || (parent is KtValueArgument && parent.getArgumentName() != null)
    }

    private fun passesParameters(call: KtCallExpression, names: List<String>): Boolean {
        if (call.lambdaArguments.isNotEmpty() || call.typeArgumentList != null) return false
        val arguments = call.valueArguments
        if (arguments.any { argument -> argument.getArgumentName() != null || argument.getSpreadElement() != null }) {
            return false
        }
        val passed = arguments.map { argument ->
            (argument.getArgumentExpression() as? KtNameReferenceExpression)?.getReferencedName()
        }
        return passed == names
    }

    private fun isStableReceiver(receiver: KtNameReferenceExpression): Boolean {
        return analyze(receiver) {
            when (val symbol = receiver.mainReference.resolveToSymbol()) {
                is KaNamedClassSymbol, is KaValueParameterSymbol -> true
                is KaLocalVariableSymbol -> symbol.isVal && (symbol.psi as? KtProperty)?.hasDelegate() == false
                is KaKotlinPropertySymbol -> {
                    if (!symbol.isVal || symbol.modality != KaSymbolModality.FINAL) return@analyze false
                    if (symbol.isExtension || symbol.getter?.isNotDefault != false) return@analyze false
                    symbol.psi is KtParameter || (symbol.psi as? KtProperty)?.hasDelegate() == false
                }
                else -> false
            }
        }
    }

    private fun findQualifiedReference(body: KtDotQualifiedExpression, names: List<String>): Reference? {
        val receiver = body.receiverExpression as? KtNameReferenceExpression ?: return null
        val receiverName = receiver.getReferencedName()
        val selector = body.selectorExpression
        val callee = (selector as? KtCallExpression)?.calleeExpression?.text
        if (receiverName == names.firstOrNull()) {
            if (selector is KtNameReferenceExpression && names.size == 1) {
                val isResolvedKotlinProperty = analyze(selector) {
                    val symbol = selector.mainReference.resolveToSymbol()
                    symbol != null && symbol !is KaSyntheticJavaPropertySymbol
                }
                return Reference("<Type>::${selector.getReferencedName()}", null)
                    .takeIf { isResolvedKotlinProperty }
            }
            if (selector !is KtCallExpression || !passesParameters(selector, names.drop(1))) return null
            return Reference("<Type>::$callee", selector)
        }
        if (receiverName in names || selector !is KtCallExpression) return null
        if (!passesParameters(selector, names)) return null
        if (!isStableReceiver(receiver)) return null
        if (receiverName == RESULT) return Reference("::$callee (import kotlin.Result.Companion.$callee)", selector)
        return Reference("$receiverName::$callee", selector)
    }

    private fun findReference(body: KtExpression, names: List<String>): Reference? {
        if (body is KtDotQualifiedExpression) return findQualifiedReference(body, names)
        if (body !is KtCallExpression || names.isEmpty() || !passesParameters(body, names)) return null
        val callee = body.calleeExpression as? KtNameReferenceExpression ?: return null
        if (analyze(callee) { callee.mainReference.resolveToSymbol() is KaVariableSymbol }) return null
        return Reference("::${body.calleeExpression?.text}", body)
    }

    private fun isAmbiguous(call: KtCallExpression?): Boolean {
        val callee = call?.calleeExpression?.text ?: return false
        return (functionCounts[callee] ?: 0) > 1
    }

    private fun isMemberExtension(call: KtCallExpression?): Boolean {
        if (call == null) return false
        return analyze(call) {
            val symbol = call.resolveToCall()?.singleFunctionCallOrNull()?.symbol ?: return@analyze false
            symbol.isExtension && symbol.callableId?.classId != null
        }
    }

    private fun hasCompatibleArity(call: KtCallExpression?): Boolean {
        if (call == null) return true
        return analyze(call) {
            val symbol = call.resolveToCall()?.singleFunctionCallOrNull()?.symbol ?: return@analyze false
            if (symbol.valueParameters.size != call.valueArguments.size) return@analyze false
            symbol.valueParameters.none { parameter -> parameter.isVararg }
        }
    }

    private fun mayBeSuspend(lambda: KtLambdaExpression, call: KtCallExpression?): Boolean {
        if (call == null) return false
        val isSuspend = analyze(call) {
            val symbol = call.resolveToCall()?.singleFunctionCallOrNull()?.symbol
            symbol is KaNamedFunctionSymbol && symbol.isSuspend
        }
        if (!isSuspend) return false
        return analyze(lambda) { (lambda.expectedType as? KaFunctionType)?.isSuspend != true }
    }

    override fun visitKtFile(file: KtFile) {
        val functions = file.collectDescendantsOfType<KtNamedFunction>()
        functionCounts = functions.mapNotNull(KtNamedFunction::getName).groupingBy { name -> name }.eachCount()
        super.visitKtFile(file)
    }

    override fun visitLambdaExpression(lambdaExpression: KtLambdaExpression) {
        super.visitLambdaExpression(lambdaExpression)
        val parameters = lambdaExpression.valueParameters
        if (parameters.any { parameter -> parameter.destructuringDeclaration != null }) return
        val names = parameters.mapNotNull(KtParameter::getName)
        if (names.isEmpty() && !isCallback(lambdaExpression)) return
        val body = lambdaExpression.bodyExpression?.statements?.singleOrNull() ?: return
        val reference = findReference(body, names) ?: return
        if (isAmbiguous(reference.call) || !hasCompatibleArity(reference.call)) return
        if (mayBeSuspend(lambdaExpression, reference.call)) return
        if (isMemberExtension(reference.call)) return
        if (reference.call == null && lambdaExpression.isSuspend()) return
        val message = "Replace the lambda with the function reference ${reference.text}: the lambda only passes " +
            "its parameters on."
        report(Finding(Entity.from(lambdaExpression), message))
    }

    private class Reference(val text: String, val call: KtCallExpression?)

    private companion object {
        const val RESULT = "Result"

        const val DESCRIPTION = "A lambda that only passes its parameters to one function is a function reference."
    }
}
