package ru.astrainteractive.klibs.rules.konsist.check.direction.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.frameworkRegistrationTypes
import ru.astrainteractive.klibs.rules.konsist.check.model.util.isRequiredOverrideReference
import ru.astrainteractive.klibs.rules.konsist.check.model.util.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.sourceReferences
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Placement
import ru.astrainteractive.klibs.rules.konsist.layout.util.describe

/**
 * Checks the direction table of `arch-context.md` through imports and fully qualified references between kinds.
 * Interfaces in `component`, `viewmodel` and `usecase` are contract targets; concrete classes retain their kind,
 * and a use case still cannot call another use case through its interface.
 * A screen is used only by what hosts or opens it: another screen, a component's `Render()`, a router, and `di`.
 *
 * Imports outside the unit are checked only for framework registration interfaces, which stay outside the domain.
 */
internal class KindImportDirection : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun allowedTargets(from: Kind): Set<Kind> {
        return when (from) {
            Kind.MODEL -> setOf(Kind.MODEL)
            Kind.CONTRACT -> setOf(Kind.MODEL)
            Kind.USECASE -> setOf(
                Kind.MODEL,
                Kind.CONTRACT,
                Kind.UTIL,
                Kind.ROLE
            )
            Kind.FAKE -> setOf(
                Kind.CONTRACT,
                Kind.MODEL,
                Kind.UTIL
            )
            Kind.UTIL -> setOf(
                Kind.MODEL,
                Kind.UTIL,
                Kind.THEME
            )
            Kind.IMPLEMENTATION -> setOf(
                Kind.MODEL,
                Kind.CONTRACT,
                Kind.UTIL,
                Kind.IMPLEMENTATION,
                Kind.ROLE
            )
            Kind.ENTRY_POINT -> setOf(
                Kind.MODEL,
                Kind.CONTRACT,
                Kind.UTIL,
                Kind.USECASE,
                Kind.THEME,
                Kind.ROLE
            )
            Kind.THEME -> setOf(Kind.MODEL, Kind.THEME)
            Kind.ROLE -> setOf(
                Kind.MODEL,
                Kind.UTIL,
                Kind.ROLE
            )
            Kind.DI -> Kind.entries.toSet()
        }
    }

    private fun isScreen(placement: Placement): Boolean {
        return placement.kindWord == Kind.COMPOSABLE_WORD && placement.subKind == SCREEN_SUB_KIND
    }

    private fun isRouterImplementation(from: Placement, fromKind: Kind): Boolean {
        return fromKind == Kind.IMPLEMENTATION && ROUTER_CONCEPT in from.concepts
    }

    private fun isUiKind(placement: Placement): Boolean {
        return placement.kind == Kind.ENTRY_POINT && placement.kindWord !in PLATFORM_INVOKED_WORDS
    }

    private fun isScreenHost(from: Placement, fromKind: Kind): Boolean {
        if (fromKind == Kind.DI || isScreen(from) || isRouterImplementation(from, fromKind)) return true
        return from.subKind == null && from.kindWord == COMPONENT_WORD
    }

    private fun isAllowedUiImport(from: Placement, fromKind: Kind, to: Placement): Boolean {
        if (!isUiKind(to)) return false
        return isUiKind(from) || isRouterImplementation(from, fromKind)
    }

    private fun isTechnologyStorage(sources: UnitSources, to: Placement): Boolean {
        if (sources.kind != UnitKind.TECHNOLOGY) return false
        val segments = to.concepts + listOfNotNull(to.kindWord) + to.below
        return segments.any { segment -> segment in TECHNOLOGY_STORAGE_WORDS }
    }

    private fun findScreenMessage(from: Placement, fromKind: Kind, to: Placement, importedName: String): String? {
        if (!isScreen(to) || isScreenHost(from, fromKind)) return null
        return "${from.describe()} must not import $importedName from ${to.describe()}: a screen is used only by " +
            "what hosts or opens it, another screen, a component's Render(), a router or di."
    }

    private fun findKindMessage(
        sources: UnitSources,
        from: Placement,
        fromKind: Kind,
        to: Placement,
        importedName: String
    ): String? {
        val toKind = to.kind ?: return null
        if (toKind in allowedTargets(fromKind)) return null
        if (fromKind == Kind.UTIL && isTechnologyStorage(sources, to)) return null
        val isContractData = sources.contractDataTypes.any { name ->
            importedName == name || importedName.startsWith("$name.")
        }
        if (fromKind == Kind.UTIL && isContractData) return null
        val isUiImport = isAllowedUiImport(
            from,
            fromKind,
            to
        )
        if (isUiImport) return null
        val kindName = fromKind.name.lowercase().replace('_', ' ')
        val article = if (kindName.first() in VOWELS) "an" else "a"
        val hasRoleKinds = sources.layout.vocabulary.roleKinds.isNotEmpty()
        val listedTargets = if (hasRoleKinds) allowedTargets(fromKind) else allowedTargets(fromKind) - Kind.ROLE
        val allowed = listedTargets.joinToString { kind -> kind.name.lowercase().replace('_', ' ') }
        return "${from.describe()} must not import $importedName from ${to.describe()}: " +
            "$article $kindName kind depends on $allowed only."
    }

    private fun checkFile(
        sources: UnitSources,
        file: KoFileDeclaration,
        projectInterfaces: Set<String>,
        references: Set<String>
    ): List<Violation> {
        val from = sources.placementOf(file) ?: return emptyList()
        val fromKind = from.kind ?: return emptyList()
        return references.mapNotNull { importedName ->
            if (fromKind == Kind.FAKE && file.isRequiredOverrideReference(importedName, projectInterfaces)) {
                return@mapNotNull null
            }
            val targetPackage = sources.packageOfImport(importedName) ?: return@mapNotNull null
            val placement = sources.layout.placementOf(targetPackage) ?: return@mapNotNull null
            val isContractHome = placement.kindWord in STATE_HOLDER_WORDS || placement.kind == Kind.USECASE
            val isUsecaseCall = fromKind == Kind.USECASE && placement.kind == Kind.USECASE
            val to = if (isContractHome && !isUsecaseCall && importedName in projectInterfaces) {
                placement.copy(kind = Kind.CONTRACT)
            } else {
                placement
            }
            val message = findScreenMessage(
                from,
                fromKind,
                to,
                importedName
            ) ?: findKindMessage(
                sources,
                from,
                fromKind,
                to,
                importedName
            ) ?: return@mapNotNull null
            sources.createViolation(
                RULE,
                file,
                importedName,
                message
            )
        }
    }

    override fun check(sources: UnitSources): List<Violation> {
        val registrationInterfaces = (sources.mainFiles + sources.dependencyFiles)
            .filter { file ->
                val word = file.packageName.substringAfterLast('.')
                Kind.fromSegment(word, sources.layout.vocabulary) == Kind.ENTRY_POINT
            }
            .flatMap { file -> file.interfaces(includeNested = false) }
            .filter { declaration -> declaration.frameworkRegistrationTypes().isNotEmpty() }
            .mapNotNull { declaration -> declaration.fullyQualifiedName }
            .toSet()
        val projectInterfaces = sources.mainFiles.flatMap { file ->
            file.interfaces(includeNested = true).mapNotNull { declaration -> declaration.fullyQualifiedName }
        }.toSet()
        val testFakes = sources.testFiles.filter { file -> sources.placementOf(file)?.kind == Kind.FAKE }
        return sources.mainFiles.plus(testFakes).flatMap { file ->
            val kind = sources.placementOf(file)?.kind
            val references = file.sourceReferences()
            val registrations = references.intersect(registrationInterfaces)
            val violations = if (kind in DOMAIN_KINDS) {
                registrations.map { reference ->
                    sources.createViolation(
                        RULE,
                        file,
                        reference,
                        "A domain model, contract or use case must not depend on framework registration $reference."
                    )
                }
            } else {
                emptyList()
            }
            val registrationHandled = kind in DOMAIN_KINDS || kind in REGISTRATION_CONSUMERS
            val checkedReferences = if (registrationHandled) references - registrations else references
            violations + checkFile(
                sources,
                file,
                projectInterfaces,
                checkedReferences
            )
        }
    }

    private companion object {
        @JvmField
        val DOMAIN_KINDS = setOf(
            Kind.MODEL,
            Kind.CONTRACT,
            Kind.USECASE
        )

        @JvmField
        val REGISTRATION_CONSUMERS = setOf(
            Kind.ENTRY_POINT,
            Kind.IMPLEMENTATION,
            Kind.DI
        )

        const val RULE = "KindImportDirection"
        const val SCREEN_SUB_KIND = "screen"

        const val COMPONENT_WORD = "component"
        const val VOWELS = "aeiou"
        const val ROUTER_CONCEPT = "router"

        @JvmField
        val STATE_HOLDER_WORDS = setOf("component", "viewmodel")

        @JvmField
        val PLATFORM_INVOKED_WORDS = setOf(
            "command",
            "error",
            "event",
            "expansion",
            "job",
            "migration",
            "route",
            "service",
            "warmup"
        )

        @JvmField
        val TECHNOLOGY_STORAGE_WORDS = setOf(
            "table",
            "entity",
            "database"
        )
    }
}
