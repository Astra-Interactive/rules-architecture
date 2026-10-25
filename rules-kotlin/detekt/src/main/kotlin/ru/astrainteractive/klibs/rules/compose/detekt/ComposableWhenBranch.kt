package ru.astrainteractive.klibs.rules.compose.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtWhenEntry
import org.jetbrains.kotlin.psi.psiUtil.getStrictParentOfType

/**
 * Reports a `when` branch of a `@Composable` function that spans more than one line (a block, a layout, a call
 * that wraps): the branch is one call of its own `@Composable`, so the `when` reads as a dispatch table.
 */
internal class ComposableWhenBranch(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitWhenEntry(jetWhenEntry: KtWhenEntry) {
        super.visitWhenEntry(jetWhenEntry)
        val branch = jetWhenEntry.expression ?: return
        if ('\n' !in branch.text) return
        val function = jetWhenEntry.getStrictParentOfType<KtNamedFunction>() ?: return
        if (!function.hasAnnotation(COMPOSABLE)) return
        val message = "Make this branch one call of its own @Composable, so the when reads as a dispatch table."
        report(Finding(Entity.from(jetWhenEntry), message))
    }

    private companion object {
        const val COMPOSABLE = "Composable"
        const val DESCRIPTION = "A when branch in a composable is one line: the call of one function."
    }
}
