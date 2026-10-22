package ru.astrainteractive.klibs.rules.konsist.check.declaration

import com.lemonappdev.konsist.api.KoModifier
import com.lemonappdev.konsist.api.provider.KoNameProvider
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a data class of a technology package that has the same primary-constructor parameter names (two or
 * more) as a data class of a `model` package of the unit: one representation per thing.
 */
internal class DuplicateRepresentation : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun dataClasses(sources: UnitSources, kind: Kind): List<DataClassShape> {
        return sources.mainFiles
            .filter { file -> sources.placementOf(file)?.kind == kind }
            .flatMap { file ->
                file.topLevelClasses()
                    .filter { declaration -> declaration.hasModifier(KoModifier.DATA) }
                    .map { declaration ->
                        val names = declaration.primaryConstructor?.parameters.orEmpty()
                            .map(KoNameProvider::name)
                            .toSet()
                        DataClassShape(
                            declaration.name,
                            names,
                            sources.relativePath(file)
                        )
                    }
            }
            .filter { shape -> shape.parameterNames.size >= MIN_PARAMETERS }
    }

    override fun check(sources: UnitSources): List<Violation> {
        val models = dataClasses(sources, Kind.MODEL)
        return dataClasses(sources, Kind.IMPLEMENTATION).mapNotNull { technology ->
            val model = models.firstOrNull { model -> model.parameterNames == technology.parameterNames }
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

    private class DataClassShape(val name: String, val parameterNames: Set<String>, val path: String)

    private companion object {
        const val RULE = "DuplicateRepresentation"
        const val MIN_PARAMETERS = 2
    }
}
