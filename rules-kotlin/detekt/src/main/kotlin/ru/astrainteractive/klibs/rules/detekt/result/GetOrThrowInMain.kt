package ru.astrainteractive.klibs.rules.detekt.result

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression

/**
 * Reports `getOrThrow()` outside the test sources (`test`, `testFixtures` and any `…Test` source set): it is for
 * tests, startup validation and APIs that require exceptions; suppress it where throwing is the intended contract.
 */
internal class GetOrThrowInMain(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        if (expression.calleeExpression?.text != GET_OR_THROW || expression.valueArguments.isNotEmpty()) return
        val path = expression.containingKtFile.virtualFilePath.replace('\\', '/')
        if (TEST_SOURCE_SET.containsMatchIn(path)) return
        val message = "Handle the failure with fold, getOrElse or onFailure instead of getOrThrow(); suppress this " +
            "only where throwing is the intended contract, such as startup validation."
        report(Finding(Entity.from(expression), message))
    }

    private companion object {
        const val GET_OR_THROW = "getOrThrow"

        @JvmField
        val TEST_SOURCE_SET = Regex("""(^|/)(test|testFixtures|[A-Za-z0-9]*Test)/""")
        const val DESCRIPTION = "getOrThrow() is used only in tests and where throwing is the intended contract."
    }
}
