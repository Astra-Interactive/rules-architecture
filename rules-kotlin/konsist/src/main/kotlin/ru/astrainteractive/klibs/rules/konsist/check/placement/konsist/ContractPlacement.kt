package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.declaration.KoInterfaceDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a contract or an implementation in the wrong place: a `*Repository` interface outside an `api`
 * package, an implementation of a contract inside `api` instead of its technology's package, and a package
 * named `network`, `local` or `remote`, which are not technology names.
 */
internal class ContractPlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun checkFile(sources: UnitSources, file: KoFileDeclaration, contractNames: Set<String>): List<Violation> {
        val kind = sources.placementOf(file)?.kind
        val repositories = file.interfaces(includeNested = false)
            .filter { declaration -> declaration.name.endsWith(REPOSITORY_SUFFIX) }
            .filterNot { kind == Kind.CONTRACT }
            .map { declaration ->
                val message = "Move ${declaration.name} to the api package of the concept it stores."
                sources.createViolation(
                    RULE,
                    file,
                    declaration,
                    declaration.name,
                    message
                )
            }
        val implementations = file.topLevelClasses()
            .filter { kind == Kind.CONTRACT }
            .filter { declaration -> declaration.parents().any { parent -> parent.name in contractNames } }
            .map { declaration ->
                val message = "Move ${declaration.name} out of api, into the package of the technology it uses."
                sources.createViolation(
                    RULE,
                    file,
                    declaration,
                    declaration.name,
                    message
                )
            }
        return repositories + implementations
    }

    private fun vagueTechnologyPackages(sources: UnitSources): List<Violation> {
        return sources.mainFiles
            .filter { file -> file.packageName.substringAfterLast('.') in VAGUE_TECHNOLOGIES }
            .distinctBy(KoFileDeclaration::packageName)
            .map { file ->
                val segment = file.packageName.substringAfterLast('.')
                val message = "Rename the package '$segment' after the library it uses: " +
                    "network, local and remote are not technology names."
                sources.createViolation(
                    RULE,
                    file,
                    file.packageName,
                    message
                )
            }
    }

    override fun check(sources: UnitSources): List<Violation> {
        val contractNames = sources.mainFiles
            .filter { file -> sources.placementOf(file)?.kind == Kind.CONTRACT }
            .flatMap { file -> file.interfaces(includeNested = false) }
            .map(KoInterfaceDeclaration::name)
            .toSet()
        val contractViolations = sources.mainFiles.flatMap { file ->
            checkFile(
                sources,
                file,
                contractNames
            )
        }
        return contractViolations + vagueTechnologyPackages(sources)
    }

    private companion object {
        const val RULE = "ContractPlacement"
        const val REPOSITORY_SUFFIX = "Repository"

        @JvmField
        val VAGUE_TECHNOLOGIES = setOf(
            "network",
            "local",
            "remote"
        )
    }
}
