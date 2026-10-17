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
 */
internal class KindShape : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

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
        if (placement.below.isEmpty() || kind == Kind.IMPLEMENTATION) return null
        val packagePath = (placement.concepts + listOfNotNull(placement.kindWord) + placement.below).joinToString("/")
        return "Move ${file.baseName} out of '$packagePath': a kind package holds only files."
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles.mapNotNull { file ->
            val placement = sources.placementOf(file) ?: return@mapNotNull null
            val message = findMessage(sources, file, placement) ?: return@mapNotNull null
            Violation(RULE, sources.relativePath(file), file.baseName, message)
        }
    }

    private companion object {
        const val RULE = "KindShape"
    }
}
