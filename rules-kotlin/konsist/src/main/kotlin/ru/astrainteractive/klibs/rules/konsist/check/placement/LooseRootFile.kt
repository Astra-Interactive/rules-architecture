package ru.astrainteractive.klibs.rules.konsist.check.placement

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.isFacade
import ru.astrainteractive.klibs.rules.konsist.check.model.isUseCase
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.Layer
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a declaration at the root of a context or a module that is neither a use case, nor the
 * `<Context>Api`, nor its `Default<Context>Api`: every other file belongs in a role package.
 */
internal class LooseRootFile : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = setOf(UnitKind.CONTEXT)

    private fun looseNames(file: KoFileDeclaration): List<String> {
        val classes = file.topLevelClasses()
            .filterNot { declaration -> declaration.isUseCase() || declaration.isFacade() }
            .map { declaration -> declaration.name }
        val interfaces = file.interfaces(includeNested = false)
            .filterNot { declaration -> declaration.name.endsWith(API_SUFFIX) }
            .map { declaration -> declaration.name }
        val others = file.objects(includeNested = false).map { declaration -> declaration.name } +
            file.functions(includeNested = false, includeLocal = false).map { declaration -> declaration.name } +
            file.properties(includeNested = false).map { declaration -> declaration.name } +
            file.typeAliases.map { declaration -> declaration.name }
        return classes + interfaces + others
    }

    override fun check(sources: UnitSources): List<Violation> = sources.mainFiles
        .filter { file -> sources.placementOf(file)?.layer == Layer.USE_CASE }
        .flatMap { file ->
            looseNames(file).map { name ->
                val message = "Move $name to a role package: the context root holds only use cases, " +
                    "the <Context>Api and its Default<Context>Api."
                Violation(RULE, sources.relativePath(file), name, message)
            }
        }

    private companion object {
        const val RULE = "LooseRootFile"
        const val API_SUFFIX = "Api"
    }
}
