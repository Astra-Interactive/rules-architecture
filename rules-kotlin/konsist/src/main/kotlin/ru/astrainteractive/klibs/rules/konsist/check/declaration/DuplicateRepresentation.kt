package ru.astrainteractive.klibs.rules.konsist.check.declaration

import com.lemonappdev.konsist.api.KoModifier
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.simpleName
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a technology data class with the same constructor field names and types as a model data class, including
 * a one-field copy: one representation per thing.
 */
internal class DuplicateRepresentation : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private data class FieldShape(val name: String, val type: String)

    private class DataClassShape(val name: String, val fields: Set<FieldShape>, val path: String)

    private fun dataClasses(sources: UnitSources, kind: Kind): List<DataClassShape> {
        return sources.mainFiles
            .filter { file -> sources.placementOf(file)?.kind == kind }
            .flatMap { file ->
                file.topLevelClasses()
                    .filter { declaration -> declaration.hasModifier(KoModifier.DATA) }
                    .map { declaration ->
                        val fields = declaration.primaryConstructor?.parameters.orEmpty()
                            .map { parameter -> FieldShape(parameter.name, parameter.type.simpleName) }
                            .toSet()
                        DataClassShape(
                            declaration.name,
                            fields,
                            sources.relativePath(file)
                        )
                    }
            }
            .filter { shape -> shape.fields.isNotEmpty() }
    }

    override fun check(sources: UnitSources): List<Violation> {
        val models = dataClasses(sources, Kind.MODEL)
        return dataClasses(sources, Kind.IMPLEMENTATION).mapNotNull { technology ->
            val model = models.firstOrNull { model -> model.fields == technology.fields }
                ?: return@mapNotNull null
            val message = "${technology.name} copies the fields of ${model.name}: one representation per thing; " +
                "keep a technology shape only when the technology dictates it, and map at its edge."
            Violation(
                RULE,
                technology.path,
                technology.name,
                message
            )
        }
    }

    private companion object {
        const val RULE = "DuplicateRepresentation"
    }
}
