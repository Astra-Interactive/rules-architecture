package ru.astrainteractive.klibs.rules.konsist.check.model

import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.declaration.KoImportDeclaration
import com.lemonappdev.konsist.api.declaration.type.KoTypeDeclaration
import java.io.File

internal val KoFileDeclaration.packageName: String
    get() = packagee?.name.orEmpty()

/**
 * The import as written: Konsist names a wildcard import by its package alone, this keeps the `.*`.
 */
internal val KoImportDeclaration.importedName: String
    get() = if (isWildcard) "$name.*" else name

internal val KoFileDeclaration.baseName: String
    get() = File(path).nameWithoutExtension

internal fun KoFileDeclaration.topLevelClasses(): List<KoClassDeclaration> =
    classes(includeNested = false, includeLocal = false)

internal fun KoFileDeclaration.topLevelClassAndObjectNames(): List<String> {
    return topLevelClasses().map { declaration -> declaration.name } +
        objects(includeNested = false).map { declaration -> declaration.name }
}

/**
 * Names of the classes, interfaces and objects at the top level of the file.
 */
internal fun KoFileDeclaration.topLevelTypeNames(): List<String> {
    return topLevelClassAndObjectNames() + interfaces(includeNested = false).map { declaration -> declaration.name }
}

/**
 * Names of the declarations at the top level of the file: classes, interfaces, objects, functions, properties
 * and type aliases.
 */
internal fun KoFileDeclaration.topLevelNames(): List<String> {
    return topLevelTypeNames() +
        functions(includeNested = false, includeLocal = false).map { declaration -> declaration.name } +
        properties(includeNested = false).map { declaration -> declaration.name } +
        typeAliases.map { declaration -> declaration.name }
}

/**
 * The simple name of the type without nullability, type arguments or qualifier: `Jail` for `model.Jail?`.
 */
internal val KoTypeDeclaration.simpleName: String
    get() = text.removeSuffix("?").substringBefore('<').substringAfterLast('.')
