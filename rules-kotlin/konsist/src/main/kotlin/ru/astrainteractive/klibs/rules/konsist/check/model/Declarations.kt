package ru.astrainteractive.klibs.rules.konsist.check.model

import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.declaration.KoImportDeclaration
import com.lemonappdev.konsist.api.declaration.type.KoTypeDeclaration
import com.lemonappdev.konsist.api.provider.KoNameProvider
import com.lemonappdev.konsist.api.provider.KoTextProvider
import java.io.File

private val NON_CODE = Regex(
    """\"\"\"[\s\S]*?\"\"\"|//[^\n]*|/\*[\s\S]*?\*/|\"(?:\\.|[^\"\\])*\"|'(?:\\.|[^'\\])*'"""
)
private val QUALIFIED_REFERENCE = Regex("""\b(?:[a-z_]\w*\.)+[A-Za-z_]\w*(?:\.[A-Za-z_]\w*)*""")
private val TYPE_NAME = Regex("""\b[A-Z]\w*\b""")

internal val KoFileDeclaration.qualifiedReferences: Set<String>
    get() {
        val declarationsText = declarations().filterIsInstance<KoTextProvider>()
            .joinToString(separator = "\n", transform = KoTextProvider::text)
        return QUALIFIED_REFERENCE.findAll(declarationsText.replace(NON_CODE, "")).map(MatchResult::value).toSet()
    }

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
 * The simple name of the type without nullability, type arguments or qualifier: `Jail` for `model.Jail?`.
 */
internal val KoTypeDeclaration.simpleName: String
    get() = text.removeSuffix("?").substringBefore('<').substringAfterLast('.')

internal val KoTypeDeclaration.typeNames: Set<String>
    get() = TYPE_NAME.findAll(text).map(MatchResult::value).toSet()

internal fun KoFileDeclaration.sourceReferences(): Set<String> {
    val wildcardPackages = imports.filter(KoImportDeclaration::isWildcard).map(KoImportDeclaration::name)
    val qualified = qualifiedReferences.filterNot { reference ->
        wildcardPackages.any(reference::startsWith)
    }
    return imports.map(KoImportDeclaration::importedName).plus(qualified).toSet()
}

internal fun KoFileDeclaration.topLevelClasses(): List<KoClassDeclaration> {
    return classes(includeNested = false, includeLocal = false)
}

internal fun KoFileDeclaration.topLevelClassAndObjectNames(): List<String> {
    return topLevelClasses().map(KoNameProvider::name) +
        objects(includeNested = false).map(KoNameProvider::name)
}

/**
 * Names of the classes, interfaces and objects at the top level of the file.
 */
internal fun KoFileDeclaration.topLevelTypeNames(): List<String> {
    return topLevelClassAndObjectNames() + interfaces(includeNested = false).map(KoNameProvider::name)
}

/**
 * Names of the declarations at the top level of the file: classes, interfaces, objects, functions, properties
 * and type aliases.
 */
internal fun KoFileDeclaration.topLevelNames(): List<String> {
    return topLevelTypeNames() +
        functions(includeNested = false, includeLocal = false).map(KoNameProvider::name) +
        properties(includeNested = false).map(KoNameProvider::name) +
        typeAliases.map(KoNameProvider::name)
}
