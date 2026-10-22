package ru.astrainteractive.klibs.rules.detekt.util

import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtElement
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.psiUtil.parents

/**
 * The property named [name] declared in the body of a class or object enclosing this element, or at the top
 * level of its file; the innermost wins.
 */
internal fun KtElement.findEnclosingProperty(name: String): KtProperty? {
    val classProperties = parents
        .filterIsInstance<KtClassOrObject>()
        .flatMap { owner -> owner.declarations.filterIsInstance<KtProperty>() }
    val fileProperties = containingKtFile.declarations.filterIsInstance<KtProperty>()
    return (classProperties + fileProperties).firstOrNull { property -> property.name == name }
}
