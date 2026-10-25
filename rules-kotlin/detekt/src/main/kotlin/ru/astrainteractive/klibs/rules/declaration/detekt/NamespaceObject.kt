package ru.astrainteractive.klibs.rules.declaration.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtObjectDeclaration

/**
 * Reports an `object` that only groups functions: no supertype, no state, nothing but functions in its body.
 * Companion objects and object expressions are not checked, nor a `<Component>Defaults` object, the public API
 * of a design-system component in Compose (`ButtonDefaults.colors()`).
 */
internal class NamespaceObject(config: Config) : Rule(config, DESCRIPTION) {

    private fun holdsOnlyFunctions(declaration: KtObjectDeclaration): Boolean {
        val members = declaration.declarations
        if (members.isEmpty()) return false
        return members.all { member -> member is KtNamedFunction }
    }

    private fun isPublicDefaults(declaration: KtObjectDeclaration): Boolean {
        val isPublic = listOf(KtTokens.PRIVATE_KEYWORD, KtTokens.INTERNAL_KEYWORD).none(declaration::hasModifier)
        val componentName = declaration.name.orEmpty().removeSuffix(DEFAULTS_SUFFIX)
        val componentFiles = setOf("$componentName.kt", "${componentName}Composable.kt")
        val hasDefaultsName = declaration.name.orEmpty().endsWith(DEFAULTS_SUFFIX)
        val isComponentFile = declaration.containingKtFile.name in componentFiles
        return isPublic && hasDefaultsName && isComponentFile
    }

    override fun visitObjectDeclaration(declaration: KtObjectDeclaration) {
        super.visitObjectDeclaration(declaration)
        if (declaration.isCompanion() || declaration.isObjectLiteral()) return
        if (declaration.superTypeListEntries.isNotEmpty()) return
        if (isPublicDefaults(declaration)) return
        if (!holdsOnlyFunctions(declaration)) return
        val message = "Make the functions of '${declaration.name}' top-level functions in a file named after them."
        report(Finding(Entity.from(declaration), message))
    }

    private companion object {
        const val DESCRIPTION = "Helpers are top-level functions, never an object used as a namespace."
        const val DEFAULTS_SUFFIX = "Defaults"
    }
}
