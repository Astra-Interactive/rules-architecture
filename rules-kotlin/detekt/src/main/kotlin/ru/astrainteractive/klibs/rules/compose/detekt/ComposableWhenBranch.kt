package ru.astrainteractive.klibs.rules.compose.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.psi.KtBlockExpression
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtExpression
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtQualifiedExpression
import org.jetbrains.kotlin.psi.KtWhenEntry
import org.jetbrains.kotlin.psi.psiUtil.getStrictParentOfType

/**
 * Reports a `when` branch of a `@Composable` function that spans more than one line (a layout, a call that wraps, a
 * block of one statement): the branch is one call of its own `@Composable`, so the `when` reads as a dispatch
 * table. Two shapes are not reported: a block of several statements, which the Compose rule allows in braces, and
 * a block whose only statement is a call of three or more arguments, which `ArgumentPerLine` wraps; the same call
 * outside braces is asked to take them.
 */
internal class ComposableWhenBranch(config: Config) : Rule(config, DESCRIPTION) {

    private fun KtExpression.asCall(): KtCallExpression? {
        return when (this) {
            is KtCallExpression -> this
            is KtQualifiedExpression -> selectorExpression as? KtCallExpression
            else -> null
        }
    }

    private fun KtExpression.isWrappedCall(): Boolean {
        val call = asCall() ?: return false
        return call.valueArguments.size >= WRAPPED_CALL_ARGUMENTS
    }

    override fun visitWhenEntry(jetWhenEntry: KtWhenEntry) {
        super.visitWhenEntry(jetWhenEntry)
        val branch = jetWhenEntry.expression ?: return
        if ('\n' !in branch.text) return
        val statements = (branch as? KtBlockExpression)?.statements
        if (statements != null && statements.size >= MIN_BLOCK_STATEMENTS) return
        if (statements?.singleOrNull()?.isWrappedCall() == true) return
        val function = jetWhenEntry.getStrictParentOfType<KtNamedFunction>() ?: return
        if (!function.hasAnnotation(COMPOSABLE)) return
        val message = if (statements == null && branch.isWrappedCall()) {
            "Put this call in braces: a branch whose call takes three or more arguments is a block `-> { }` " +
                "holding that call alone."
        } else {
            "Make this branch one call of its own @Composable, so the when reads as a dispatch table."
        }
        report(Finding(Entity.from(jetWhenEntry), message))
    }

    private companion object {
        const val COMPOSABLE = "Composable"
        const val MIN_BLOCK_STATEMENTS = 2
        const val WRAPPED_CALL_ARGUMENTS = 3
        const val DESCRIPTION = "A when branch in a composable is one line: the call of one function."
    }
}
