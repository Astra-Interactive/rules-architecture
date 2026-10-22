package ru.astrainteractive.klibs.rules.detekt.compose

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtFile
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtObjectDeclaration
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports the case of the values of a composable or `theme` file: a `const val` is `UPPER_SNAKE`, any other
 * `val` (a `Dp`, a `Color`, a model instance) is `PascalCase`. Top-level values are checked, and in a `theme`
 * package also the values of its top-level objects.
 */
internal class ComposeConstantCase(config: Config) : Rule(config, DESCRIPTION) {

    private fun isThemeFile(file: KtFile): Boolean {
        val packageName = file.packageFqName
        return !packageName.isRoot && packageName.shortName().asString() == THEME_SEGMENT
    }

    private fun checkedProperties(file: KtFile, isTheme: Boolean): List<KtProperty> {
        val topLevel = file.declarations.filterIsInstance<KtProperty>()
        if (!isTheme) return topLevel
        val objectMembers = file.declarations
            .filterIsInstance<KtObjectDeclaration>()
            .flatMap { declaration -> declaration.declarations.filterIsInstance<KtProperty>() }
        return topLevel + objectMembers
    }

    private fun findMessage(property: KtProperty): String? {
        val name = property.name ?: return null
        if (property.isVar) return null
        if (property.hasModifier(KtTokens.CONST_KEYWORD)) {
            if (UPPER_SNAKE.matches(name)) return null
            return "Name the constant '$name' in UPPER_SNAKE case: a const val of a composable or theme file is " +
                "UPPER_SNAKE."
        }
        if (PASCAL.matches(name)) return null
        return "Name the value '$name' in PascalCase: a val object of a composable or theme file (a Dp, a Color, " +
            "a model instance) is PascalCase."
    }

    override fun visitKtFile(file: KtFile) {
        super.visitKtFile(file)
        val isTheme = isThemeFile(file)
        val hasComposable = file.collectDescendantsOfType<KtNamedFunction>()
            .any { function -> function.hasAnnotation(COMPOSABLE) }
        if (!isTheme && !hasComposable) return
        checkedProperties(file, isTheme).forEach { property ->
            val message = findMessage(property) ?: return@forEach
            report(Finding(Entity.atName(property), message))
        }
    }

    private companion object {
        const val COMPOSABLE = "Composable"
        const val THEME_SEGMENT = "theme"

        @JvmField
        val UPPER_SNAKE = Regex("[A-Z][A-Z0-9_]*")

        @JvmField
        val PASCAL = Regex("[A-Z][A-Za-z0-9]*")
        const val DESCRIPTION = "In composable and theme files a const val is UPPER_SNAKE and a val is PascalCase."
    }
}
