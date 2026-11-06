package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import com.lemonappdev.konsist.api.KoModifier
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.provider.KoFullyQualifiedNameProvider
import com.lemonappdev.konsist.api.provider.KoNameProvider
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.packageName
import java.io.File

/**
 * Reports a class or object whose direct supertype is a sealed class or interface declared in another file of the
 * unit: a sealed hierarchy is one file with all its variants. A supertype is resolved as Kotlin resolves it, so two
 * outer types that each nest a sealed `Model` hold two families.
 */
internal class SealedFamilyFile : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun sealedFiles(files: List<KoFileDeclaration>): Map<String, KoFileDeclaration> {
        return files.flatMap { file ->
            val classes = file.classes(includeNested = true, includeLocal = false)
                .filter { declaration -> declaration.hasModifier(KoModifier.SEALED) }
                .mapNotNull(KoFullyQualifiedNameProvider::fullyQualifiedName)
            val interfaces = file.interfaces(includeNested = true)
                .filter { declaration -> declaration.hasModifier(KoModifier.SEALED) }
                .mapNotNull(KoFullyQualifiedNameProvider::fullyQualifiedName)
            classes.plus(interfaces).map { name -> SealedType(name, file) }
        }.associateBy(keySelector = SealedType::name, valueTransform = SealedType::file)
    }

    private fun findTypeNames(files: List<KoFileDeclaration>): Set<String> {
        return files.flatMap { file ->
            val classes = file.classes(includeNested = true, includeLocal = false)
                .mapNotNull(KoFullyQualifiedNameProvider::fullyQualifiedName)
            val interfaces = file.interfaces(includeNested = true)
                .mapNotNull(KoFullyQualifiedNameProvider::fullyQualifiedName)
            val objects = file.objects(includeNested = true)
                .mapNotNull(KoFullyQualifiedNameProvider::fullyQualifiedName)
            classes + interfaces + objects
        }.toSet()
    }

    /**
     * Resolves [written], a supertype of the declaration named [qualifiedName] in [file], in Kotlin's order: the types
     * nested in each enclosing class, innermost first, then the explicit imports, then the file's package.
     */
    private fun resolve(
        written: String,
        qualifiedName: String,
        file: KoFileDeclaration,
        typeNames: Set<String>
    ): String {
        val head = written.substringBefore('.')
        val tail = written.removePrefix(head)
        val nested = generateSequence(qualifiedName) { name -> name.substringBeforeLast('.', "") }
            .drop(1)
            .takeWhile { scope -> scope.length > file.packageName.length }
            .map { scope -> "$scope.$head" }
            .firstOrNull(typeNames::contains)
        val imported = file.imports.firstOrNull { declaration ->
            val visibleName = declaration.alias?.name ?: declaration.name.substringAfterLast('.')
            visibleName == head
        }?.name
        val packaged = listOf(file.packageName, head).filter(String::isNotEmpty).joinToString(".")
        return (nested ?: imported ?: packaged) + tail
    }

    override fun check(sources: UnitSources): List<Violation> {
        val sealedByName = sealedFiles(sources.mainFiles)
        val typeNames = findTypeNames(sources.mainFiles)
        return sources.mainFiles.flatMap { file ->
            val classes = file.classes(includeNested = true, includeLocal = false)
            val objects = file.objects(includeNested = true)
            classes.plus(objects).mapNotNull { declaration ->
                val qualifiedName = declaration.fullyQualifiedName.orEmpty()
                val sealedFile = declaration.parents()
                    .map(KoNameProvider::name)
                    .map { name ->
                        resolve(
                            name.substringBefore('<'),
                            qualifiedName,
                            file,
                            typeNames
                        )
                    }
                    .firstNotNullOfOrNull { name -> sealedByName[name]?.takeIf { sealed -> sealed.path != file.path } }
                    ?: return@mapNotNull null
                val message = "Move ${declaration.name} into ${File(sealedFile.path).name}: a sealed hierarchy is " +
                    "one file with all its variants."
                sources.createViolation(
                    RULE,
                    file,
                    declaration,
                    declaration.name,
                    message
                )
            }
        }
    }

    private class SealedType(val name: String, val file: KoFileDeclaration)

    private companion object {
        const val RULE = "SealedFamilyFile"
    }
}
