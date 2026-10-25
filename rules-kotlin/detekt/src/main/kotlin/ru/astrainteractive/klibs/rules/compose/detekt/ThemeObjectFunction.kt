package ru.astrainteractive.klibs.rules.compose.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtObjectDeclaration
import org.jetbrains.kotlin.psi.psiUtil.containingClassOrObject

/**
 * Reports a function declared in an object of a `theme` package: a theme object holds the values several
 * composables share, never functions.
 */
internal class ThemeObjectFunction(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitNamedFunction(function: KtNamedFunction) {
        super.visitNamedFunction(function)
        val owner = function.containingClassOrObject as? KtObjectDeclaration ?: return
        val packageName = function.containingKtFile.packageFqName
        if (packageName.isRoot || packageName.shortName().asString() != THEME_SEGMENT) return
        val message = "Move '${function.name}' out of the theme object '${owner.name}': a theme object holds " +
            "values, not functions; a function goes to util/ or into the composable that uses it."
        report(Finding(Entity.atName(function), message))
    }

    private companion object {
        const val THEME_SEGMENT = "theme"
        const val DESCRIPTION = "A theme object holds values, not functions."
    }
}
