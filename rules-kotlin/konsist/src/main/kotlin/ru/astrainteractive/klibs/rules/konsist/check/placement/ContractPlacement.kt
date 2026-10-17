package ru.astrainteractive.klibs.rules.konsist.check.placement

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a contract or an implementation in the wrong place: a `*Repository` interface outside an `api`
 * package, an implementation of a contract inside `api` instead of its technology's package, and a package
 * named `network`, `local` or `remote`, which are not technology names.
 */
internal class ContractPlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun checkFile(sources: UnitSources, file: KoFileDeclaration, contractNames: Set<String>): List<Violation> {
        val kind = sources.placementOf(file)?.kind
        val path = sources.relativePath(file)
        val repositories = file.interfaces(includeNested = false)
            .filter { declaration -> declaration.name.endsWith(REPOSITORY_SUFFIX) }
            .filterNot { kind == Kind.CONTRACT }
            .map { declaration ->
                val message = "Move ${declaration.name} to the api package of the concept it stores."
                Violation(RULE, path, declaration.name, message)
            }
        val implementations = file.topLevelClasses()
            .filter { kind == Kind.CONTRACT }
            .filter { declaration -> declaration.parents().any { parent -> parent.name in contractNames } }
            .map { declaration ->
                val message = "Move ${declaration.name} out of api, into the package of the technology it uses."
                Violation(RULE, path, declaration.name, message)
            }
        return repositories + implementations
    }

    private fun vagueTechnologyPackages(sources: UnitSources): List<Violation> {
        return sources.mainFiles
            .filter { file -> file.packageName.substringAfterLast('.') in VAGUE_TECHNOLOGIES }
            .distinctBy { file -> file.packageName }
            .map { file ->
                val segment = file.packageName.substringAfterLast('.')
                val message = "Rename the package '$segment' after the library it uses: " +
                    "network, local and remote are not technology names."
                Violation(RULE, sources.relativePath(file), file.packageName, message)
            }
    }

    override fun check(sources: UnitSources): List<Violation> {
        val contractNames = sources.mainFiles
            .filter { file -> sources.placementOf(file)?.kind == Kind.CONTRACT }
            .flatMap { file -> file.interfaces(includeNested = false) }
            .map { declaration -> declaration.name }
            .toSet()
        return sources.mainFiles.flatMap { file -> checkFile(sources, file, contractNames) } +
            vagueTechnologyPackages(sources)
    }

    private companion object {
        const val RULE = "ContractPlacement"
        const val REPOSITORY_SUFFIX = "Repository"

        @JvmField
        val VAGUE_TECHNOLOGIES = setOf("network", "local", "remote")
    }
}
