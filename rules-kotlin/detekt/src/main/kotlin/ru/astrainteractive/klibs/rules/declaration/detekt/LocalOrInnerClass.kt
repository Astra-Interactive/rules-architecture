package ru.astrainteractive.klibs.rules.declaration.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtClass

/**
 * Reports a class declared inside a function body and an `inner class`.
 */
internal class LocalOrInnerClass(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitClass(klass: KtClass) {
        super.visitClass(klass)
        if (klass.isLocal) {
            report(Finding(Entity.from(klass), "Move '${klass.name}' out of the function body."))
        }
        if (klass.hasModifier(KtTokens.INNER_KEYWORD)) {
            val message = "Make '${klass.name}' a nested class that receives what it needs through its constructor."
            report(Finding(Entity.from(klass), message))
        }
    }

    private companion object {
        const val DESCRIPTION = "A class is never declared inside a function body and is never an inner class."
    }
}
