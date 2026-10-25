package ru.astrainteractive.klibs.rules.result.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import ru.astrainteractive.klibs.rules.detekt.util.isTestSource

/**
 * Reports `getOrThrow()` outside the test sources (`test`, `testFixtures` and any `…Test` source set): it is for
 * tests, startup validation and APIs that require exceptions; suppress it where throwing is the intended contract.
 */
internal class GetOrThrowInMain(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitCallExpression(expression: KtCallExpression) {
        super.visitCallExpression(expression)
        if (expression.calleeExpression?.text != GET_OR_THROW || expression.valueArguments.isNotEmpty()) return
        if (expression.containingKtFile.isTestSource()) return
        val message = "Handle the failure with fold, getOrElse or onFailure instead of getOrThrow(); suppress this " +
            "only where throwing is the intended contract, such as startup validation."
        report(Finding(Entity.from(expression), message))
    }

    private companion object {
        const val GET_OR_THROW = "getOrThrow"
        const val DESCRIPTION = "getOrThrow() is used only in tests and where throwing is the intended contract."
    }
}
