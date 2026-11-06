package ru.astrainteractive.klibs.rules.konsist.check.model.util

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.declaration.KoInterfaceDeclaration
import com.lemonappdev.konsist.api.provider.KoAnnotationProvider
import ru.astrainteractive.klibs.rules.core.technology.util.isTechnologyName
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Vocabulary

internal val FRAMEWORK_PREFIXES: List<String> = listOf(
    "android.",
    "androidx.",
    "com.velocitypowered.",
    "io.ktor.",
    "io.papermc.",
    "net.fabricmc.",
    "net.md_5.",
    "net.minecraft.",
    "net.minecraftforge.",
    "net.neoforged.",
    "org.bukkit.",
    "org.jetbrains.exposed.",
    "org.jetbrains.compose.",
    "org.spigotmc."
)

private val SIMPLE_TYPE = Regex("""[A-Za-z_]\w*(?:\.[A-Za-z_]\w*)*""")

private fun KoInterfaceDeclaration.countModelProperties(files: List<KoFileDeclaration>, visited: Set<String>): Int? {
    val qualifiedName = "${containingFile.packageName}.$name"
    if (qualifiedName in visited) return null
    if (functions(includeNested = false).isNotEmpty()) return null
    val hasBehavior = objects(includeNested = false).any { declaration ->
        if (!declaration.hasCompanionModifier) return@any true
        if (declaration.functions(includeNested = false).isNotEmpty()) return@any true
        declaration.properties(includeNested = false).any { property -> !property.hasConstModifier }
    }
    if (hasBehavior) return null
    val properties = properties(includeNested = false)
    if (properties.any { property -> !property.isVal || property.isInitialized || property.getter != null }) return null
    if (properties.any { property -> property.type?.text?.contains("->") == true }) return null
    val interfaces = files.flatMap { file -> file.interfaces(includeNested = false) }
    var count = properties.size
    for (parent in parents()) {
        val parentName = containingFile.resolveTypeName(parent.name.substringBefore('<'))
        val declaration = interfaces.singleOrNull { candidate ->
            "${candidate.containingFile.packageName}.${candidate.name}" == parentName
        } ?: return null
        if (declaration.containingFile.packageName.substringAfterLast('.') != "model") return null
        count += declaration.countModelProperties(files, visited + qualifiedName) ?: return null
    }
    return count
}

internal fun KoInterfaceDeclaration.isDataShape(kind: Kind?, files: List<KoFileDeclaration>): Boolean {
    if (kind != Kind.MODEL) return false
    return (countModelProperties(files, emptySet()) ?: return false) > 0
}

/**
 * Recognises abstract, synchronous registration signatures in a mechanism package.
 * Unknown parents and types do not establish a framework boundary.
 */
internal fun KoInterfaceDeclaration.frameworkRegistrationTypes(): Set<String> {
    if (parents().isNotEmpty() || properties(includeNested = false).isNotEmpty()) return emptySet()
    val functions = functions(includeNested = false)
    if (functions.isEmpty()) return emptySet()
    val references = mutableSetOf<String>()
    for (function in functions) {
        if (function.hasSuspendModifier || function.hasReturnValue || function.isInitialized) return emptySet()
        if (function.parameters.isEmpty()) return emptySet()
        for (parameter in function.parameters) {
            val typeName = parameter.type.text.trim().removeSuffix("?")
            if (!SIMPLE_TYPE.matches(typeName)) return emptySet()
            val reference = containingFile.resolveTypeName(typeName)
            if (FRAMEWORK_PREFIXES.none(reference::startsWith)) return emptySet()
            references.add(reference)
        }
    }
    return references
}

internal fun KoInterfaceDeclaration.isFrameworkRegistration(unitKind: UnitKind, kind: Kind?): Boolean {
    val platformFree = setOf(
        UnitKind.API,
        UnitKind.CORE_API,
        UnitKind.STORAGE_CONTRACT,
        UnitKind.CONTRACT
    )
    if (unitKind in platformFree) return false
    if (kind != Kind.ENTRY_POINT) return false
    return frameworkRegistrationTypes().isNotEmpty()
}

private fun KoAnnotationProvider.hasTechnologyAnnotation(file: KoFileDeclaration, vocabulary: Vocabulary): Boolean {
    return annotations.any { annotation ->
        val reference = file.resolveTypeName(annotation.text.removePrefix("@").substringBefore('(').trim())
        val isFramework = FRAMEWORK_PREFIXES.any(reference::startsWith)
        val technologySegments = reference.split('.').dropLast(1).filter { segment ->
            isTechnologyName(segment) || segment in vocabulary.technologyWords
        }
        isFramework && technologySegments.isNotEmpty()
    }
}

/**
 * Recognises an interface that its technology implements with generated code: it, or every function it declares,
 * carries an annotation of a known framework named after a technology of the unit (Room's `@Dao`, or `@Insert` and
 * `@Delete` on a base of DAOs). Such an interface is the technology's own type, not a domain contract.
 */
internal fun KoInterfaceDeclaration.isTechnologyBinding(vocabulary: Vocabulary): Boolean {
    if (hasTechnologyAnnotation(containingFile, vocabulary)) return true
    val functions = functions(includeNested = false)
    return functions.isNotEmpty() && functions.all { function ->
        function.hasTechnologyAnnotation(containingFile, vocabulary)
    }
}
