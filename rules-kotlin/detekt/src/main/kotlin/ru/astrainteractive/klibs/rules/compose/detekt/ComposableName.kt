package ru.astrainteractive.klibs.rules.compose.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtNamedFunction

/**
 * Reports a `@Composable` that emits UI, a PascalCase function returning `Unit`, whose name does not end in
 * `Composable`. A function of the preview block (`…Preview`, `OrderSummaryPreviewSheet`), the theme function in a
 * `theme` package (`AppTheme`) and an override such as a component's `Render()` keep their names.
 */
internal class ComposableName(config: Config) : Rule(config, DESCRIPTION) {

    private fun emitsUi(function: KtNamedFunction, name: String): Boolean {
        val isPascalCase = name.first().isUpperCase()
        val returnType = function.typeReference?.text
        return isPascalCase && (returnType == null || returnType == UNIT)
    }

    private fun isExempt(function: KtNamedFunction, name: String): Boolean {
        if (PREVIEW in name || function.hasModifier(KtTokens.OVERRIDE_KEYWORD)) return true
        val packageName = function.containingKtFile.packageFqName
        return !packageName.isRoot && packageName.shortName().asString() == THEME_SEGMENT
    }

    override fun visitNamedFunction(function: KtNamedFunction) {
        super.visitNamedFunction(function)
        val name = function.name ?: return
        if (!function.hasAnnotation(COMPOSABLE) || !emitsUi(function, name)) return
        if (name.endsWith(COMPOSABLE) || isExempt(function, name)) return
        val message = "Name '$name' for what it shows, ending in Composable ('${name}Composable'): a composable that " +
            "emits UI is named <Thing>Composable, a screen <Feature>ScreenComposable."
        report(Finding(Entity.atName(function), message))
    }

    private companion object {
        const val COMPOSABLE = "Composable"
        const val PREVIEW = "Preview"
        const val UNIT = "Unit"
        const val THEME_SEGMENT = "theme"
        const val DESCRIPTION = "A @Composable that emits UI is named for what it shows and ends in Composable."
    }
}
