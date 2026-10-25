package ru.astrainteractive.klibs.rules.konsist.check.util

import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.model.typeNames

/**
 * The type names of the properties this class declares, generic arguments included.
 */
internal fun KoClassDeclaration.propertyTypeNames(): Set<String> {
    val constructorTypes = primaryConstructor?.parameters.orEmpty()
        .filter { parameter -> parameter.isVal || parameter.isVar }
        .flatMap { parameter -> parameter.type.typeNames }
    val bodyTypes = properties(includeNested = false)
        .flatMap { property -> property.type?.typeNames.orEmpty() }
    return constructorTypes.plus(bodyTypes).toSet()
}

/**
 * The paths of the files whose text names [name] as a whole word.
 */
internal fun List<KoFileDeclaration>.pathsNaming(name: String): Set<String> {
    val word = Regex("\\b${Regex.escape(name)}\\b")
    return filter { file -> word.containsMatchIn(file.text) }.map(KoFileDeclaration::path).toSet()
}
