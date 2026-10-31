package ru.astrainteractive.klibs.rules.testing.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.psi.KtBlockExpression
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtReturnExpression
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports a `@Test` that calls `runTest` without returning it as a `TestResult`: on JS a test must return what
 * `runTest` returns, so the test reads `fun GIVEN_…(): TestResult { return runTest { … } }`, with the return type and
 * the `return` written out; an expression body (`= runTest { … }`) is reported too.
 */
internal class RunTestResult(config: Config) : Rule(config, DESCRIPTION) {

    private fun isRunTest(expression: KtExpression?): Boolean {
        return (expression as? KtCallExpression)?.calleeExpression?.text == RUN_TEST
    }

    private fun returnsRunTest(function: KtNamedFunction): Boolean {
        val body = function.bodyExpression as? KtBlockExpression ?: return false
        val statement = body.statements.singleOrNull() as? KtReturnExpression ?: return false
        return isRunTest(statement.returnedExpression)
    }

    override fun visitNamedFunction(function: KtNamedFunction) {
        super.visitNamedFunction(function)
        if (!function.hasAnnotation(TEST_ANNOTATION)) return
        val body = function.bodyExpression ?: return
        val callsRunTest = body.collectDescendantsOfType<KtCallExpression>().any(::isRunTest)
        if (!callsRunTest) return
        val isTestResult = function.typeReference?.text == TEST_RESULT
        if (isTestResult && returnsRunTest(function)) return
        val message = "Return runTest from '${function.name}': declare ': TestResult' and write " +
            "'return runTest { … }', since a test on JS must return what runTest returns."
        report(Finding(Entity.atName(function), message))
    }

    private companion object {
        const val TEST_ANNOTATION = "Test"
        const val RUN_TEST = "runTest"
        const val TEST_RESULT = "TestResult"
        const val DESCRIPTION = "A coroutine test returns runTest as its TestResult."
    }
}
