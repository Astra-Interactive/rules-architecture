package ru.astrainteractive.klibs.rules.lambda.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.lexer.KtTokens
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
import org.jetbrains.kotlin.psi.psiUtil.parentsWithSelf
import ru.astrainteractive.klibs.rules.detekt.util.findCallName

/**
 * Reports a lambda whose whole body is one call that receives the lambda's parameters unchanged and in order
 * (`{ p -> foo(p) }`, `{ p -> Foo(p) }`, `{ p -> p.foo() }`, `{ p -> p.prop }`, `{ p -> obj.foo(p) }`), and a
 * callback `{ obj.foo() }` passed as a named argument or stored in a property: a function reference says the same.
 *
 * Works on syntax, so it stays silent where a reference may not compile or may mean another function: in
 * coroutine code (a `suspend` function or a coroutine builder's lambda), for a callee declared `suspend` in the
 * file or declared there more than once, and for type, named, spread or trailing-lambda arguments.
 */
internal class TrivialLambdaReference(config: Config) : Rule(config, DESCRIPTION) {
    private var functionCounts: Map<String, Int> = emptyMap()
    private var suspendNames: Set<String> = emptySet()

    private fun isInCoroutine(lambda: KtLambdaExpression): Boolean {
        for (parent in lambda.parentsWithSelf) {
            if (parent is KtLambdaExpression && parent.findCallName() in COROUTINE_CALLS) return true
            if (parent is KtNamedFunction) return parent.hasModifier(KtTokens.SUSPEND_KEYWORD)
        }
        return false
    }

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

    private fun findQualifiedReference(body: KtDotQualifiedExpression, names: List<String>): Reference? {
        val receiver = body.receiverExpression as? KtNameReferenceExpression ?: return null
        val receiverName = receiver.getReferencedName()
        val selector = body.selectorExpression
        val callee = (selector as? KtCallExpression)?.calleeExpression?.text
        if (receiverName == names.firstOrNull()) {
            if (selector is KtNameReferenceExpression && names.size == 1) {
                return Reference("<Type>::${selector.getReferencedName()}", null)
            }
            if (selector !is KtCallExpression || !passesParameters(selector, names.drop(1))) return null
            return Reference("<Type>::$callee", callee)
        }
        if (receiverName in names || selector !is KtCallExpression) return null
        if (!passesParameters(selector, names)) return null
        return Reference("$receiverName::$callee", callee)
    }

    private fun findReference(body: KtExpression, names: List<String>): Reference? {
        if (body is KtDotQualifiedExpression) return findQualifiedReference(body, names)
        if (body !is KtCallExpression || names.isEmpty() || !passesParameters(body, names)) return null
        val callee = body.calleeExpression as? KtNameReferenceExpression ?: return null
        return Reference("::${callee.getReferencedName()}", callee.getReferencedName())
    }

    private fun isAmbiguous(callee: String?): Boolean {
        if (callee == null) return false
        return (functionCounts[callee] ?: 0) > 1 || callee in suspendNames
    }

    override fun visitKtFile(file: KtFile) {
        val functions = file.collectDescendantsOfType<KtNamedFunction>()
        functionCounts = functions.mapNotNull(KtNamedFunction::getName).groupingBy { name -> name }.eachCount()
        suspendNames = functions
            .filter { function -> function.hasModifier(KtTokens.SUSPEND_KEYWORD) }
            .mapNotNull(KtNamedFunction::getName)
            .toSet()
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
        if (isAmbiguous(reference.callee) || isInCoroutine(lambdaExpression)) return
        val message = "Replace the lambda with the function reference ${reference.text}: the lambda only passes " +
            "its parameters on."
        report(Finding(Entity.from(lambdaExpression), message))
    }

    private class Reference(val text: String, val callee: String?)

    private companion object {
        @JvmField
        val COROUTINE_CALLS = setOf(
            "launch",
            "async",
            "withContext",
            "coroutineScope",
            "supervisorScope",
            "flow",
            "collect",
            "collectLatest",
            "runTest",
            "LaunchedEffect"
        )
        const val DESCRIPTION = "A lambda that only passes its parameters to one function is a function reference."
    }
}
