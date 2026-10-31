package ru.astrainteractive.klibs.rules.konsist.check.direction.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.sourceReferences
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Placement
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.layout.util.describe

/**
 * Checks the direction table of `arch-context.md` through imports and fully qualified references between kinds:
 * `model` on `model`, contracts on `model`, use cases on contracts, `model` and `util`, implementations on those and
 * their own technology packages, entry points on use cases, contracts, `model` and `util`, `theme` on `model`, and
 * nothing on `di`; `util` and the entry points also reach `theme`, and `util` of a one-technology unit its `table`,
 * `entity` and `database`. Entry points import each other only between UI kinds (`component`, `composable`, `gui`,
 * `slot`, the project's own words), and a router's implementation reaches the UI kinds it opens.
 *
 * A screen is used only by what hosts or opens it: another screen, a component's `Render()`, a router, and `di`.
 *
 * Imports that leave the unit, and packages of no known kind, are not judged.
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
                Kind.UTIL
            )
            Kind.UTIL -> setOf(
                Kind.MODEL,
                Kind.CONTRACT,
                Kind.UTIL,
                Kind.THEME
            )
            Kind.IMPLEMENTATION -> setOf(
                Kind.MODEL,
                Kind.CONTRACT,
                Kind.UTIL,
                Kind.IMPLEMENTATION
            )
            Kind.ENTRY_POINT -> setOf(
                Kind.MODEL,
                Kind.CONTRACT,
                Kind.UTIL,
                Kind.USECASE,
                Kind.THEME
            )
            Kind.THEME -> setOf(Kind.MODEL, Kind.THEME)
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
        val isUiImport = isAllowedUiImport(
            from,
            fromKind,
            to
        )
        if (isUiImport) return null
        val kindName = fromKind.name.lowercase().replace('_', ' ')
        val article = if (kindName.first() in VOWELS) "an" else "a"
        val allowed = allowedTargets(fromKind).joinToString { kind -> kind.name.lowercase().replace('_', ' ') }
        return "${from.describe()} must not import $importedName from ${to.describe()}: " +
            "$article $kindName kind depends on $allowed only."
    }

    private fun checkFile(sources: UnitSources, file: KoFileDeclaration): List<Violation> {
        val from = sources.placementOf(file) ?: return emptyList()
        val fromKind = from.kind ?: return emptyList()
        return file.sourceReferences().mapNotNull { importedName ->
            val targetPackage = sources.packageOfImport(importedName) ?: return@mapNotNull null
            val to = sources.layout.placementOf(targetPackage) ?: return@mapNotNull null
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
        return sources.mainFiles.flatMap { file -> checkFile(sources, file) }
    }

    private companion object {
        const val RULE = "KindImportDirection"
        const val SCREEN_SUB_KIND = "screen"

        const val COMPONENT_WORD = "component"
        const val VOWELS = "aeiou"
        const val ROUTER_CONCEPT = "router"

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
