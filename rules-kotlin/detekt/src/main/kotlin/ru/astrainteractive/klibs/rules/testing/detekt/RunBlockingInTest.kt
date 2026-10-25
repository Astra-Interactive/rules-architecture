package ru.astrainteractive.klibs.rules.testing.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import ru.astrainteractive.klibs.rules.detekt.util.isTestSource

/**
 * Reports `runBlocking` in a test source set: coroutine tests run in `runTest` from `kotlinx.coroutines.test`.
 */
internal class RunBlockingInTest(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        if (expression.calleeExpression?.text != RUN_BLOCKING) return
        if (!expression.containingKtFile.isTestSource()) return
        val message = "Replace runBlocking with runTest from kotlinx.coroutines.test: it skips delays and reports " +
            "coroutines that leak from the test."
        report(Finding(Entity.from(expression), message))
    }

    private companion object {
        const val RUN_BLOCKING = "runBlocking"
        const val DESCRIPTION = "Coroutine tests use runTest, never runBlocking."
    }
}
