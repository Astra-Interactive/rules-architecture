package ru.astrainteractive.klibs.rules.konsist.check.model

import com.lemonappdev.konsist.api.KoModifier
import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.declaration.KoImportDeclaration
import com.lemonappdev.konsist.api.declaration.type.KoTypeDeclaration
import java.io.File

private val FACADE_NAME = Regex("Default\\w+Api")

internal val KoFileDeclaration.packageName: String
    get() = packagee?.name.orEmpty()

/**
 * The import as written: Konsist names a wildcard import by its package alone, this keeps the `.*`.
 */
internal val KoImportDeclaration.importedName: String
    get() = if (isWildcard) "$name.*" else name

internal val KoFileDeclaration.baseName: String
    get() = File(path).nameWithoutExtension

/**
 * Names of the declarations at the top level of the file: classes, interfaces, objects, functions, properties
 * and type aliases.
 */
internal fun KoFileDeclaration.topLevelNames(): List<String> =
    classes(includeNested = false, includeLocal = false).map { declaration -> declaration.name } +
        interfaces(includeNested = false).map { declaration -> declaration.name } +
        objects(includeNested = false).map { declaration -> declaration.name } +
        functions(includeNested = false, includeLocal = false).map { declaration -> declaration.name } +
        properties(includeNested = false).map { declaration -> declaration.name } +
        typeAliases.map { declaration -> declaration.name }

internal fun KoFileDeclaration.topLevelClasses(): List<KoClassDeclaration> =
    classes(includeNested = false, includeLocal = false)

/**
 * A use case is a class whose entry is `operator fun invoke`.
 */
internal fun KoClassDeclaration.isUseCase(): Boolean =
    functions(includeNested = false, includeLocal = false).any { function ->
        function.name == "invoke" && function.hasModifier(KoModifier.OPERATOR)
    }

/**
 * The one implementation of a context's published interface, `Default<Context>Api`.
 */
internal fun KoClassDeclaration.isFacade(): Boolean = FACADE_NAME.matches(name)

/**
 * The simple name of the type without nullability, type arguments or qualifier: `Jail` for `model.Jail?`.
 */
internal val KoTypeDeclaration.simpleName: String
    get() = text.removeSuffix("?").substringBefore('<').substringAfterLast('.')
