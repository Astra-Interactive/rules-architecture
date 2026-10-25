package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.provider.KoNameProvider
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a use case or a repository in `core`, a unit whose Gradle path has the segment `core`: `core` holds
 * config, translation and permission models, shared helpers and scopes, never a use case or a repository.
 */
internal class CoreShape : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    override fun check(sources: UnitSources): List<Violation> {
        val isCore = sources.unitPath.orEmpty().split(':').contains(CORE)
        if (!isCore) return emptyList()
        return sources.mainFiles.flatMap { file ->
            val useCases = file.topLevelClasses()
                .map(KoNameProvider::name)
                .filter { name -> name.endsWith(USE_CASE_SUFFIX) }
            val repositories = file.interfaces(includeNested = false)
                .map(KoNameProvider::name)
                .filter { name -> name.endsWith(REPOSITORY_SUFFIX) }
            useCases.plus(repositories).map { name ->
                val message = "Move $name out of core: a use case goes to the api unit of the sub-context that " +
                    "owns the operation, a repository to the storage contracts unit."
                Violation(
                    RULE,
                    sources.relativePath(file),
                    name,
                    message
                )
            }
        }
    }

    private companion object {
        const val RULE = "CoreShape"
        const val CORE = "core"
        const val USE_CASE_SUFFIX = "UseCase"
        const val REPOSITORY_SUFFIX = "Repository"
    }
}
