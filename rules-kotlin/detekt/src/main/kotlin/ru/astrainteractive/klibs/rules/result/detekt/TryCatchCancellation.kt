package ru.astrainteractive.klibs.rules.result.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.api.RequiresAnalysisApi
import dev.detekt.api.Rule
import dev.detekt.rules.coroutines.SuspendFunSwallowedCancellation
import org.jetbrains.kotlin.psi.KtCatchClause
import org.jetbrains.kotlin.psi.KtFile

/**
 * Reports a `catch` in suspend code that also catches `CancellationException` without rethrowing it first: detekt's
 * `SuspendFunSwallowedCancellation` decides it, and only its `catch` findings are kept. Its `runCatching` findings
 * are dropped, because the error-handling rule writes `runCatching { … }.propagateCancellationException()` in suspend
 * code, which that rule cannot see; [RunCatchingCancellation] judges `runCatching` instead.
 */
internal class TryCatchCancellation(config: Config) : Rule(config, DESCRIPTION), RequiresAnalysisApi {
    private val swallowedCancellation = SuspendFunSwallowedCancellation(Config.empty)

    override fun visit(root: KtFile) {
        swallowedCancellation.visitFile(root, languageVersionSettings)
            .filter { finding -> finding.entity.ktElement is KtCatchClause }
            .forEach { finding -> report(Finding(finding.entity, MESSAGE)) }
    }

    private companion object {
        const val MESSAGE = "Rethrow CancellationException before handling the failure: this catch in suspend code " +
            "also catches the cancellation and breaks structured concurrency; catch the exception you expect instead."
        const val DESCRIPTION = "A catch in suspend code rethrows CancellationException before anything else."
    }
}
