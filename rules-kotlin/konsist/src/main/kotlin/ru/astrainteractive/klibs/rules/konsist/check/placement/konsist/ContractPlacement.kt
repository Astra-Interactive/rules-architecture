package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.KoModifier
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
 * Reports a contract or an implementation in the wrong place: a top-level interface outside an `api` package, an
 * implementation of a contract inside `api` instead of its technology's package, and a package named `network`,
 * `local` or `remote`, which are not technology names. A sealed interface is a model family, a use case's interface
 * sits in `usecase`, a DI module's in `di`, and a state holder's (`…Component`, `…ViewModel`) in `component` or
 * `viewmodel`, never in `api`. A Compose scope (`interface FilterCardScope : ColumnScope`) is the receiver of a
 * composable's content, not a contract.
 */
internal class ContractPlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun isComposeScope(file: KoFileDeclaration, declaration: KoInterfaceDeclaration): Boolean {
        return declaration.parents().any { parent ->
            val name = parent.name.substringBefore('<')
            name.endsWith(SCOPE_SUFFIX) && file.imports.any { import ->
                import.name.endsWith(".$name") && UnitSources.isComposeImport(import.name)
            }
        }
    }

    private fun stateHolderWord(declaration: KoInterfaceDeclaration): String? {
        return STATE_HOLDER_SUFFIXES.entries
            .firstOrNull { (suffix, _) -> declaration.name.endsWith(suffix) }
            ?.value
    }

    private fun findMessage(declaration: KoInterfaceDeclaration, kindWord: String?, isInterfaceHome: Boolean): String? {
        val stateHolderWord = stateHolderWord(declaration)
        if (stateHolderWord != null) {
            if (kindWord in STATE_HOLDER_WORDS) return null
            return "Move ${declaration.name} to the $stateHolderWord package of its concept: a state holder's " +
                "interface sits beside its Default… implementation, in the owner's api unit when another unit calls it."
        }
        if (isInterfaceHome) return null
        if (declaration.name.endsWith(REPOSITORY_SUFFIX)) {
            return "Move ${declaration.name} to the api package of the concept it stores."
        }
        return "Move ${declaration.name} to the api package of its concept: a contract sits in api."
    }

    private fun checkFile(sources: UnitSources, file: KoFileDeclaration, contractNames: Set<String>): List<Violation> {
        val placement = sources.placementOf(file)
        val kind = placement?.kind
        val isInterfaceHome = kind in INTERFACE_KINDS || placement?.kindWord in STATE_HOLDER_WORDS
        val contracts = file.interfaces(includeNested = false)
            .filterNot { declaration -> declaration.hasModifier(KoModifier.SEALED) }
            .filterNot { kind == null }
            .filterNot { declaration -> isComposeScope(file, declaration) }
            .mapNotNull { declaration ->
                val message = findMessage(
                    declaration,
                    placement?.kindWord,
                    isInterfaceHome
                ) ?: return@mapNotNull null
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
        return contracts + implementations
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
        val INTERFACE_KINDS = setOf(
            Kind.CONTRACT,
            Kind.USECASE,
            Kind.DI
        )

        const val SCOPE_SUFFIX = "Scope"

        @JvmField
        val STATE_HOLDER_WORDS = setOf("component", "viewmodel")

        @JvmField
        val STATE_HOLDER_SUFFIXES = mapOf(
            "Component" to "component",
            "ViewModel" to "viewmodel"
        )

        @JvmField
        val VAGUE_TECHNOLOGIES = setOf(
            "network",
            "local",
            "remote"
        )
    }
}
