package ru.astrainteractive.klibs.rules.result.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.RequiresAnalysisApi
import dev.detekt.api.Rule
import org.jetbrains.kotlin.analysis.api.analyze
import org.jetbrains.kotlin.analysis.api.types.symbol
import org.jetbrains.kotlin.psi.KtBlockExpression
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtFile
import org.jetbrains.kotlin.psi.KtFunctionLiteral
import org.jetbrains.kotlin.psi.KtQualifiedExpression
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports a `Result` used as a statement without consuming its success or failure. `onSuccess` and `onFailure`
 * consume it for side effects; assignments, returns and value-producing terminal calls are not statements, and
 * neither is the last expression of a lambda that returns a value (`mutex.withLock { runCatching { … } }`), which is
 * that lambda's result. In a lambda that returns `Unit` (`forEach { runCatching { … } }`) it is still discarded.
 */
internal class ResultConsumption(config: Config) : Rule(config, DESCRIPTION), RequiresAnalysisApi {
    private fun terminalCallName(statement: KtExpression): String? {
        val call = if (statement is KtCallExpression) {
            statement
        } else {
            (statement as KtQualifiedExpression).selectorExpression as? KtCallExpression
        }
        return call?.calleeExpression?.text
    }

    private fun isLambdaResult(statement: KtExpression): Boolean {
        val block = statement.parent as? KtBlockExpression ?: return false
        val lambda = block.parent as? KtFunctionLiteral ?: return false
        if (block.statements.lastOrNull() != statement) return false
        return analyze(lambda) {
            lambda.symbol.returnType.symbol?.classId?.asFqNameString() != UNIT_TYPE
        }
    }

    private fun check(statement: KtExpression) {
        if (terminalCallName(statement) in CONSUMING_CALLS || isLambdaResult(statement)) return
        val resultExpression = (statement as? KtQualifiedExpression)?.receiverExpression ?: statement
        val isResult = analyze(resultExpression) {
            resultExpression.expressionType?.symbol?.classId?.asFqNameString() == RESULT_TYPE
        }
        if (!isResult) return
        val message = "Consume this Result: handle its success and failure instead of discarding it."
        report(Finding(Entity.from(statement), message))
    }

    override fun visitKtFile(file: KtFile) {
        super.visitKtFile(file)
        file.collectDescendantsOfType<KtExpression>()
            .filter { statement -> statement.parent is KtBlockExpression }
            .filter { statement -> statement is KtCallExpression || statement is KtQualifiedExpression }
            .forEach(::check)
    }

    private companion object {
        const val RESULT_TYPE = "kotlin.Result"
        const val UNIT_TYPE = "kotlin.Unit"

        @JvmField
        val CONSUMING_CALLS = setOf(
            "exceptionOrNull",
            "fold",
            "getOrElse",
            "getOrNull",
            "getOrThrow",
            "isFailure",
            "isSuccess",
            "onFailure",
            "onSuccess"
        )
        const val DESCRIPTION = "A Result is consumed, never discarded."
    }
}
