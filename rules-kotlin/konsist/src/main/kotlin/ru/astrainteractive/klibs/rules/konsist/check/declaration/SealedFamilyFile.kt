package ru.astrainteractive.klibs.rules.konsist.check.declaration

import com.lemonappdev.konsist.api.KoModifier
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File

/**
 * Reports a class or object whose direct supertype is a sealed class or interface declared in another file of the
 * unit: a sealed hierarchy is one file with all its variants.
 */
internal class SealedFamilyFile : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun sealedFiles(files: List<KoFileDeclaration>): Map<String, KoFileDeclaration> {
        return files.flatMap { file ->
            val classes = file.classes(includeNested = true, includeLocal = false)
                .filter { declaration -> declaration.hasModifier(KoModifier.SEALED) }
                .map { declaration -> declaration.name }
            val interfaces = file.interfaces(includeNested = true)
                .filter { declaration -> declaration.hasModifier(KoModifier.SEALED) }
                .map { declaration -> declaration.name }
            (classes + interfaces).map { name -> SealedType(name, file) }
        }.associate { sealed -> sealed.name to sealed.file }
    }

    override fun check(sources: UnitSources): List<Violation> {
        val sealedByName = sealedFiles(sources.mainFiles)
        return sources.mainFiles.flatMap { file ->
            val classes = file.classes(includeNested = true, includeLocal = false)
                .map { declaration -> Subtype(declaration.name, declaration.parents().map { parent -> parent.name }) }
            val objects = file.objects(includeNested = true)
                .map { declaration -> Subtype(declaration.name, declaration.parents().map { parent -> parent.name }) }
            (classes + objects).mapNotNull { subtype ->
                val parentName = subtype.parentNames
                    .map { name -> name.substringBefore('<') }
                    .firstOrNull { name -> sealedByName[name]?.path.let { path -> path != null && path != file.path } }
                    ?: return@mapNotNull null
                val sealedFile = File(sealedByName.getValue(parentName).path).name
                val message = "Move ${subtype.name} into $sealedFile: a sealed hierarchy is one file with all its " +
                    "variants."
                Violation(
                    RULE,
                    sources.relativePath(file),
                    subtype.name,
                    message
                )
            }
        }
    }

    private class SealedType(val name: String, val file: KoFileDeclaration)

    private class Subtype(val name: String, val parentNames: List<String>)

    private companion object {
        const val RULE = "SealedFamilyFile"
    }
}
