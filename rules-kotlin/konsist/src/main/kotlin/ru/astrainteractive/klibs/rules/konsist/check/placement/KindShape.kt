package ru.astrainteractive.klibs.rules.konsist.check.placement

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.baseName
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.Placement
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Checks the shape `<unit root>[/<concept>]/<kind>/<File>`: no file lies loose in the unit root or in a concept
 * package, and a kind package holds files, not packages. A technology unit's concept packages hold its
 * implementations, technology packages may hold their own kinds (`table`), and the entry point keeps its main
 * class at the root.
 *
 * Compose's `composable` is the one kind that may hold sub-packages, one level of them, each holding files only:
 * named in the singular (`screen`, `component`, `layout`), never `model`, `util` or `theme`, and never next to
 * files of `composable` itself.
 */
internal class KindShape : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun packagePath(placement: Placement): String {
        return (placement.concepts + listOfNotNull(placement.kindWord, placement.subKind) + placement.below)
            .joinToString("/")
    }

    private fun kindPath(placement: Placement): String {
        return (placement.concepts + listOfNotNull(placement.kindWord)).joinToString("/")
    }

    private fun isPlural(segment: String): Boolean = segment.endsWith("s") && !segment.endsWith("ss")

    private fun findComposableMessage(file: KoFileDeclaration, placement: Placement, subKind: String): String? {
        val path = packagePath(placement)
        if (subKind in LEVEL_KINDS) {
            return "Move ${file.baseName} out of '$path': $subKind is a kind of the level, never a sub-package of " +
                "composable."
        }
        if (isPlural(subKind)) {
            return "Rename '$path' to the singular: a composable sub-package is named like a kind."
        }
        if (placement.below.isEmpty()) return null
        return "Move ${file.baseName} out of '$path': a composable sub-package holds only files."
    }

    private fun findMessage(sources: UnitSources, file: KoFileDeclaration, placement: Placement): String? {
        val kind = placement.kind
        if (kind == null && placement.concepts.isEmpty()) {
            if (sources.kind == UnitKind.ENTRY_POINT) return null
            return "Move ${file.baseName} into a kind package: the unit root holds only packages."
        }
        if (kind == null) {
            return "Move ${file.baseName} into a kind package of ${placement.describe()}: a concept package " +
                "holds only packages."
        }
        val subKind = placement.subKind
        if (subKind != null) return findComposableMessage(file, placement, subKind)
        if (placement.below.isEmpty() || kind == Kind.IMPLEMENTATION) return null
        return "Move ${file.baseName} out of '${packagePath(placement)}': a kind package holds only files."
    }

    private fun findMixedComposables(sources: UnitSources): List<Violation> {
        val splitComposables = sources.mainFiles
            .mapNotNull { file -> sources.placementOf(file) }
            .filter { placement -> placement.subKind != null }
            .map { placement -> kindPath(placement) }
            .toSet()
        return sources.mainFiles.mapNotNull { file ->
            val placement = sources.placementOf(file) ?: return@mapNotNull null
            val isComposableFile = placement.kindWord == Kind.COMPOSABLE_WORD && placement.subKind == null
            if (!isComposableFile || kindPath(placement) !in splitComposables) return@mapNotNull null
            val message = "Move ${file.baseName} into a sub-package of ${placement.describe()}: composable holds " +
                "files or sub-packages, never both."
            Violation(RULE, sources.relativePath(file), file.baseName, message)
        }
    }

    override fun check(sources: UnitSources): List<Violation> {
        val shapeViolations = sources.mainFiles.mapNotNull { file ->
            val placement = sources.placementOf(file) ?: return@mapNotNull null
            val message = findMessage(sources, file, placement) ?: return@mapNotNull null
            Violation(RULE, sources.relativePath(file), file.baseName, message)
        }
        return shapeViolations + findMixedComposables(sources)
    }

    private companion object {
        const val RULE = "KindShape"

        @JvmField
        val LEVEL_KINDS = setOf("model", "util", "theme")
    }
}
