package ru.astrainteractive.klibs.rules.konsist.check.placement

import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.baseName
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a main source file of an entry-point unit that is neither at the unit root nor in `di`: the entry point
 * holds the platform main class and `RootModule`, nothing else.
 */
internal class EntryPointShape : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = setOf(UnitKind.ENTRY_POINT)

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles
            .filter { file ->
                val placement = sources.placementOf(file) ?: return@filter true
                val isRoot = placement.concepts.isEmpty() && placement.kind == null
                !isRoot && placement.kind != Kind.DI
            }
            .map { file ->
                val message = "Move ${file.baseName} out of the entry point: the entry point holds the platform main " +
                    "class and RootModule, nothing else; the code goes to the unit of its sub-context or to core."
                Violation(
                    RULE,
                    sources.relativePath(file),
                    file.baseName,
                    message
                )
            }
    }

    private companion object {
        const val RULE = "EntryPointShape"
    }
}
