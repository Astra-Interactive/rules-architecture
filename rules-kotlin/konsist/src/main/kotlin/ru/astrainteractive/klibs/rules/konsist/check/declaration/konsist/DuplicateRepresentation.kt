package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import com.lemonappdev.konsist.api.KoModifier
import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.simpleName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a technology data class with the same constructor field names and types as a model data class of this
 * unit or of a unit it depends on (`data/exposed` copying `data/dao`), including a one-field copy: one
 * representation per thing.
 */
internal class DuplicateRepresentation : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private data class FieldShape(val name: String, val type: String)

    private class DataClassShape(
        val declaration: KoClassDeclaration,
        val file: KoFileDeclaration,
        val name: String,
        val fields: Set<FieldShape>
    )

    private fun dataClasses(files: List<KoFileDeclaration>): List<DataClassShape> {
        return files
            .flatMap { file ->
                file.topLevelClasses()
                    .filter { declaration -> declaration.hasModifier(KoModifier.DATA) }
                    .map { declaration ->
                        val fields = declaration.primaryConstructor?.parameters.orEmpty()
                            .map { parameter -> FieldShape(parameter.name, parameter.type.simpleName) }
                            .toSet()
                        DataClassShape(
                            declaration,
                            file,
                            declaration.name,
                            fields
                        )
                    }
            }
            .filter { shape -> shape.fields.isNotEmpty() }
    }

    override fun check(sources: UnitSources): List<Violation> {
        val ownModels = sources.mainFiles.filter { file -> sources.placementOf(file)?.kind == Kind.MODEL }
        val models = dataClasses(ownModels + sources.dependencyModelFiles)
        val technologies = sources.mainFiles.filter { file -> sources.placementOf(file)?.kind == Kind.IMPLEMENTATION }
        return dataClasses(technologies).mapNotNull { technology ->
            val model = models.firstOrNull { model -> model.fields == technology.fields }
                ?: return@mapNotNull null
            val message = "${technology.name} copies the fields of ${model.name}: one representation per thing; " +
                "keep a technology shape only when the technology dictates it, and map at its edge."
            sources.createViolation(
                RULE,
                technology.file,
                technology.declaration,
                technology.name,
                message
            )
        }
    }

    private companion object {
        const val RULE = "DuplicateRepresentation"
    }
}
