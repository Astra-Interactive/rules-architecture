package ru.astrainteractive.klibs.rules.structure.nesting.structure

import ru.astrainteractive.klibs.rules.structure.unit.api.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import java.io.File

/**
 * Reports sources in the root project, which holds none: the code lives in the units below it.
 */
public class RootSources : StructureCheck {
    override fun check(unit: BuildUnit): List<StructureViolation> {
        if (!unit.isRoot || !File(unit.directory, SOURCES).isDirectory) return emptyList()
        val message = "Move the sources of the root project into a unit: the root project holds no sources."
        return listOf(
            StructureViolation(
                RULE,
                SOURCES,
                SOURCES,
                message
            )
        )
    }

    private companion object {
        const val RULE = "RootSources"
        const val SOURCES = "src"
    }
}
