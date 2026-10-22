package ru.astrainteractive.klibs.rules.konsist.check.util

import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.model.simpleName

/**
 * The simple type names of the properties this class declares, in its primary constructor or its body.
 */
internal fun KoClassDeclaration.propertyTypeNames(): Set<String> {
    val constructorTypes = primaryConstructor?.parameters.orEmpty()
        .filter { parameter -> parameter.isVal || parameter.isVar }
        .map { parameter -> parameter.type.simpleName }
    val bodyTypes = properties(includeNested = false)
        .mapNotNull { property -> property.type?.simpleName }
    return (constructorTypes + bodyTypes).toSet()
}

/**
 * The paths of the files whose text names [name] as a whole word.
 */
internal fun List<KoFileDeclaration>.pathsNaming(name: String): Set<String> {
    val word = Regex("\\b${Regex.escape(name)}\\b")
    return filter { file -> word.containsMatchIn(file.text) }.map { file -> file.path }.toSet()
}
