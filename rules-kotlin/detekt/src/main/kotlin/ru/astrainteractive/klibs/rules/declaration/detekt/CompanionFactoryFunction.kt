package ru.astrainteractive.klibs.rules.declaration.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtObjectDeclaration
import org.jetbrains.kotlin.psi.psiUtil.containingClassOrObject

/**
 * Reports a factory in a companion object: a function named `create…`, `of` or `from`. The model is data; a
 * factory with a rule is a top-level `create<Aggregate>()` in `util` returning `Result`.
 */
internal class CompanionFactoryFunction(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitNamedFunction(function: KtNamedFunction) {
        super.visitNamedFunction(function)
        val owner = function.containingClassOrObject as? KtObjectDeclaration ?: return
        if (!owner.isCompanion()) return
        val name = function.name.orEmpty()
        if (!name.startsWith(CREATE_PREFIX) && name !in FACTORY_NAMES) return
        val message = "Move the factory '$name' out of the companion object: write a top-level " +
            "create<Aggregate>() in util returning Result, never a companion factory."
        report(Finding(Entity.atName(function), message))
    }

    private companion object {
        const val CREATE_PREFIX = "create"

        @JvmField
        val FACTORY_NAMES = setOf("of", "from")
        const val DESCRIPTION = "A factory is a top-level create<Aggregate>() in util, never a companion function."
    }
}
