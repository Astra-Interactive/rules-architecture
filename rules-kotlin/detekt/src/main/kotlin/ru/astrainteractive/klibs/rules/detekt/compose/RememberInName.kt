package ru.astrainteractive.klibs.rules.detekt.compose

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports a function named `remember…` whose body calls no `remember` or `remember…` function: `remember` in a
 * name promises `remember` in the body.
 */
internal class RememberInName(config: Config) : Rule(config, DESCRIPTION) {

    private fun callsRemember(body: KtCallExpression): Boolean {
        val name = body.calleeExpression?.text ?: return false
        return REMEMBER_NAME.matches(name)
    }

    override fun visitNamedFunction(function: KtNamedFunction) {
        super.visitNamedFunction(function)
        val name = function.name ?: return
        if (!REMEMBER_NAME.matches(name)) return
        val body = function.bodyExpression ?: return
        if (body.collectDescendantsOfType<KtCallExpression>().any(::callsRemember)) return
        val message = "'$name' calls no remember: call remember in its body, or name it with a verb such as " +
            "build, resolve or select."
        report(Finding(Entity.atName(function), message))
    }

    private companion object {
        @JvmField
        val REMEMBER_NAME = Regex("remember([A-Z].*)?")
        const val DESCRIPTION = "A function named remember… calls remember."
    }
}
