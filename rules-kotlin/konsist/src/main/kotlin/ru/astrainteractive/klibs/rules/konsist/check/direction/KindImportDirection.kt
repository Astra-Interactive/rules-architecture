package ru.astrainteractive.klibs.rules.konsist.check.direction

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.sourceReferences
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.Placement
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import ru.astrainteractive.klibs.rules.konsist.layout.util.describe

/**
 * Checks the direction table of `arch-context.md` through imports and fully qualified references between kinds:
 * `model` depends on nothing of its level, contracts on `model`, use cases on contracts and `model`,
 * implementations on the contracts they implement and `model` (and on their own `table` or `entity` package),
 * entry points on use cases and contracts, `theme` on `model`, and nothing depends on `di`. A use case also
 * reaches `util`, where the rules put the extensions it calls; `util` and the entry points also reach `theme`,
 * which only a Compose unit has. The sub-packages of `composable` import each other, except that nothing but `di`
 * depends on `composable/screen`; the screen files of one package may call each other.
 *
 * Imports that leave the unit, and packages of no known kind, are not judged.
 */
internal class KindImportDirection : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun allowedTargets(from: Kind): Set<Kind> {
        return when (from) {
            Kind.MODEL -> setOf(Kind.MODEL)
            Kind.CONTRACT -> setOf(Kind.MODEL, Kind.CONTRACT)
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
                Kind.IMPLEMENTATION
            )
            Kind.ENTRY_POINT -> setOf(
                Kind.MODEL,
                Kind.CONTRACT,
                Kind.UTIL,
                Kind.USECASE,
                Kind.ENTRY_POINT,
                Kind.THEME
            )
            Kind.THEME -> setOf(Kind.MODEL, Kind.THEME)
            Kind.DI -> Kind.entries.toSet()
        }
    }

    private fun isScreen(placement: Placement): Boolean {
        return placement.kindWord == Kind.COMPOSABLE_WORD && placement.subKind == SCREEN_SUB_KIND
    }

    private fun findScreenMessage(from: Placement, fromKind: Kind, to: Placement, importedName: String): String? {
        if (!isScreen(to) || fromKind == Kind.DI) return null
        if (isScreen(from) && from.concepts == to.concepts) return null
        return "${from.describe()} must not import $importedName from ${to.describe()}: nothing depends on a " +
            "screen but di."
    }

    private fun findKindMessage(from: Placement, fromKind: Kind, to: Placement, importedName: String): String? {
        val toKind = to.kind ?: return null
        if (toKind in allowedTargets(fromKind)) return null
        return "${from.describe()} must not import $importedName from ${to.describe()}: " +
            "a ${fromKind.name.lowercase()} kind depends on " +
            allowedTargets(fromKind).joinToString { kind -> kind.name.lowercase() } + " only."
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
                from,
                fromKind,
                to,
                importedName
            ) ?: return@mapNotNull null
            Violation(
                RULE,
                sources.relativePath(file),
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
    }
}
