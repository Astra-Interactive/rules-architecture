package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import com.lemonappdev.konsist.api.KoModifier
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.provider.KoNameProvider
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
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
                .map(KoNameProvider::name)
            val interfaces = file.interfaces(includeNested = true)
                .filter { declaration -> declaration.hasModifier(KoModifier.SEALED) }
                .map(KoNameProvider::name)
            classes.plus(interfaces).map { name -> SealedType(name, file) }
        }.associateBy(keySelector = SealedType::name, valueTransform = SealedType::file)
    }

    override fun check(sources: UnitSources): List<Violation> {
        val sealedByName = sealedFiles(sources.mainFiles)
        return sources.mainFiles.flatMap { file ->
            val classes = file.classes(includeNested = true, includeLocal = false)
            val objects = file.objects(includeNested = true)
            classes.plus(objects).mapNotNull { declaration ->
                val parentName = declaration.parents()
                    .map(KoNameProvider::name)
                    .map { name -> name.substringBefore('<') }
                    .firstOrNull { name -> sealedByName[name]?.path.let { path -> path != null && path != file.path } }
                    ?: return@mapNotNull null
                val sealedFile = File(sealedByName.getValue(parentName).path).name
                val message = "Move ${declaration.name} into $sealedFile: a sealed hierarchy is one file with all " +
                    "its variants."
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
