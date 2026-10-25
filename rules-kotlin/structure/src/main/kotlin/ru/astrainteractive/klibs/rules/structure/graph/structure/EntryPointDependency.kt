package ru.astrainteractive.klibs.rules.structure.graph.structure

import ru.astrainteractive.klibs.rules.structure.unit.api.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation

/**
 * Reports a dependency on an entry-point unit, one whose path has the segment `instances`: nothing depends on the
 * entry point. The kind is read from the path alone, since reading another project's plugins breaks project
 * isolation.
 */
public class EntryPointDependency : StructureCheck {

    override fun check(unit: BuildUnit): List<StructureViolation> {
        return unit.projectDependencies
            .sorted()
            .filter { dependency -> ENTRY_POINT_SEGMENT in dependency.split(':') }
            .map { dependency ->
                val message = "Drop the dependency on the entry point $dependency: nothing depends on the entry " +
                    "point; depend on the api unit that holds what you need."
                StructureViolation(
                    RULE,
                    BUILD_FILE,
                    dependency,
                    message
                )
            }
    }

    private companion object {
        const val RULE = "EntryPointDependency"
        const val BUILD_FILE = "build.gradle.kts"
        const val ENTRY_POINT_SEGMENT = "instances"
    }
}
