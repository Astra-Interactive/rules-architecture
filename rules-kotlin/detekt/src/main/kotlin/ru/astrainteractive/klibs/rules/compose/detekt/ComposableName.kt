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
 * `Composable`, and a screen `<Feature>ScreenComposable` in a file of another name. A function of the preview block
 * (`…Preview`, `OrderSummaryPreviewSheet`) and an override such as a component's `Render()` keep their names; the
 * theme function in a `theme` package (`AppTheme`) keeps its name and sits in `AppThemeComposable.kt`.
 */
internal class ComposableName(config: Config) : Rule(config, DESCRIPTION) {

    private fun emitsUi(function: KtNamedFunction, name: String): Boolean {
        val isPascalCase = name.first().isUpperCase()
        val returnType = function.typeReference?.text
        return isPascalCase && (returnType == null || returnType == UNIT)
    }

    private fun isThemeFunction(function: KtNamedFunction): Boolean {
        val packageName = function.containingKtFile.packageFqName
        return !packageName.isRoot && packageName.shortName().asString() == THEME_SEGMENT
    }

    private fun findThemeMessage(fileName: String, name: String): String? {
        val expectedFile = "$name$COMPOSABLE$KOTLIN_EXTENSION"
        if (fileName == expectedFile) return null
        return "Move the theme function '$name' to $expectedFile: the theme object sits alone in its own file, the " +
            "theme function with its CompositionLocals in <Theme>Composable.kt."
    }

    private fun findMessage(function: KtNamedFunction, name: String): String? {
        val fileName = function.containingKtFile.name
        if (isThemeFunction(function)) return findThemeMessage(fileName, name)
        if (PREVIEW in name || function.hasModifier(KtTokens.OVERRIDE_KEYWORD)) return null
        if (!name.endsWith(COMPOSABLE)) {
            return "Name '$name' for what it shows, ending in Composable ('${name}Composable'): a composable that " +
                "emits UI is named <Thing>Composable, a screen <Feature>ScreenComposable."
        }
        val expectedFile = "$name$KOTLIN_EXTENSION"
        if (!name.endsWith(SCREEN_SUFFIX) || fileName == expectedFile) return null
        return "Move the screen '$name' to $expectedFile: a screen composable's file carries its name."
    }

    override fun visitNamedFunction(function: KtNamedFunction) {
        super.visitNamedFunction(function)
        val name = function.name ?: return
        if (!function.hasAnnotation(COMPOSABLE) || !emitsUi(function, name)) return
        val message = findMessage(function, name) ?: return
        report(Finding(Entity.atName(function), message))
    }

    private companion object {
        const val COMPOSABLE = "Composable"
        const val SCREEN_SUFFIX = "ScreenComposable"
        const val PREVIEW = "Preview"
        const val UNIT = "Unit"
        const val THEME_SEGMENT = "theme"
        const val KOTLIN_EXTENSION = ".kt"
        const val DESCRIPTION = "A @Composable that emits UI is named for what it shows and ends in Composable."
    }
}
