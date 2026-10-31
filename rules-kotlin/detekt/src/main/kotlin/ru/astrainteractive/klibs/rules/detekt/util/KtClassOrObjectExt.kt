package ru.astrainteractive.klibs.rules.detekt.util

import org.jetbrains.kotlin.lexer.KtTokens
import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtObjectDeclaration

private const val DEFAULTS_SUFFIX = "Defaults"

/**
 * Whether this is the public `<Component>Defaults` object of a design-system component, declared in the file of
 * its composable (`GlassButtonDefaults` in `GlassButtonComposable.kt`), the one declaration a composable file holds
 * beside its functions.
 */
internal fun KtClassOrObject.isComponentDefaults(): Boolean {
    val objectName = name.orEmpty()
    val isPublic = listOf(KtTokens.PRIVATE_KEYWORD, KtTokens.INTERNAL_KEYWORD).none(::hasModifier)
    val componentName = objectName.removeSuffix(DEFAULTS_SUFFIX)
    val componentFiles = setOf("$componentName.kt", "${componentName}Composable.kt")
    val hasDefaultsName = objectName.endsWith(DEFAULTS_SUFFIX)
    val isComponentFile = containingKtFile.name in componentFiles
    return this is KtObjectDeclaration && isPublic && hasDefaultsName && isComponentFile
}
