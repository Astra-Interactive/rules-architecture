package ru.astrainteractive.klibs.rules.konsist.check.placement

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.Layer
import ru.astrainteractive.klibs.rules.konsist.layout.Placement
import ru.astrainteractive.klibs.rules.konsist.layout.Role
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a contract and its implementation on the wrong level: a `*Repository` interface outside the `data`
 * contract package, an implementation of a contract on the contract's own level instead of one level down,
 * and a technology package named `database`, `network`, `local` or `remote`.
 */
internal class ContractLevel : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = setOf(UnitKind.CONTEXT)

    private fun misplacedRepositories(file: KoFileDeclaration, placement: Placement): List<String> = file
        .interfaces(includeNested = false)
        .filter { declaration -> declaration.name.endsWith(REPOSITORY_SUFFIX) }
        .filterNot { placement.layer == Layer.CONTRACT && placement.role == Role.DATA }
        .map { declaration -> declaration.name }

    private fun implementationsOnContractLevel(
        file: KoFileDeclaration,
        placement: Placement,
        contractNames: Set<String>
    ): List<String> {
        if (placement.layer != Layer.CONTRACT) return emptyList()
        return file.topLevelClasses()
            .filter { declaration -> declaration.parents().any { parent -> parent.name in contractNames } }
            .map { declaration -> declaration.name }
    }

    private fun checkFile(sources: UnitSources, file: KoFileDeclaration, contractNames: Set<String>): List<Violation> {
        val placement = sources.placementOf(file) ?: return emptyList()
        val path = sources.relativePath(file)
        val repositories = misplacedRepositories(file, placement).map { name ->
            Violation(RULE, path, name, "Move $name to the data package of its context: it is a repository contract.")
        }
        val implementations = implementationsOnContractLevel(file, placement, contractNames).map { name ->
            val message = "Move $name one level down, into a package named by the technology it uses."
            Violation(RULE, path, name, message)
        }
        return repositories + implementations
    }

    private fun vagueTechnologyPackages(sources: UnitSources): List<Violation> = sources.mainFiles
        .filter { file -> sources.placementOf(file)?.technology in VAGUE_TECHNOLOGIES }
        .distinctBy { file -> file.packageName }
        .map { file ->
            val technology = sources.placementOf(file)?.technology
            val message = "Rename the package '$technology' after the library it uses: " +
                "database, network, local and remote are not technology names."
            Violation(RULE, sources.relativePath(file), file.packageName, message)
        }

    override fun check(sources: UnitSources): List<Violation> {
        val contractNames = sources.mainFiles
            .filter { file -> sources.placementOf(file)?.layer == Layer.CONTRACT }
            .flatMap { file -> file.interfaces(includeNested = false) }
            .map { declaration -> declaration.name }
            .toSet()
        return sources.mainFiles.flatMap { file -> checkFile(sources, file, contractNames) } +
            vagueTechnologyPackages(sources)
    }

    private companion object {
        const val RULE = "ContractLevel"
        const val REPOSITORY_SUFFIX = "Repository"

        @JvmField
        val VAGUE_TECHNOLOGIES = setOf("database", "network", "local", "remote")
    }
}
