package ru.astrainteractive.klibs.rules.konsist.check.declaration

import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a data contract named for its technology instead of `<Aggregate>Repository` (an `api` interface named
 * `…Dao`, `…Storage` or `…Krate`), and a `…Repository` interface that extends a generic base (a supertype named
 * `…Repository` or `…Crud…`).
 */
internal class ContractName : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles.flatMap { file ->
            val isContract = sources.placementOf(file)?.kind == Kind.CONTRACT
            file.interfaces(includeNested = false).mapNotNull { declaration ->
                val name = declaration.name
                val technologySuffix = TECHNOLOGY_SUFFIXES.firstOrNull(name::endsWith)
                val genericBase = declaration.parents()
                    .map { parent -> parent.name.substringBefore('<') }
                    .firstOrNull { parent -> parent.endsWith(REPOSITORY) || CRUD in parent }
                val message = when {
                    isContract && technologySuffix != null ->
                        "Rename $name to <Aggregate>Repository: $technologySuffix names a technology, not the contract."
                    name.endsWith(REPOSITORY) && genericBase != null ->
                        "Drop $genericBase from $name: a repository speaks the context's words and extends no " +
                            "generic base."
                    else -> null
                } ?: return@mapNotNull null
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
        const val RULE = "ContractName"
        const val REPOSITORY = "Repository"
        const val CRUD = "Crud"

        @JvmField
        val TECHNOLOGY_SUFFIXES = listOf(
            "Dao",
            "Storage",
            "Krate"
        )
    }
}
