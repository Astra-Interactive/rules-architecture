package ru.astrainteractive.klibs.rules.structure.graph

import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.StructureViolation

/**
 * Reports a dependency on a technology unit (`data/exposed`, `room`, `ktor`, …) from a unit that is neither the
 * entry point nor a technology unit itself: storage is reached through its contracts unit.
 */
public class TechnologyDependency : StructureCheck {

    private fun isTechnology(path: String): Boolean = path.substringAfterLast(':') in TECHNOLOGIES

    override fun check(unit: BuildUnit): List<StructureViolation> {
        if (unit.isEntryPoint || isTechnology(unit.path)) return emptyList()
        return unit.projectDependencies
            .sorted()
            .filter(::isTechnology)
            .map { dependency ->
                val message = "Depend on the contracts unit instead of the technology unit $dependency: only the " +
                    "technology unit and the entry point depend on the technology."
                StructureViolation(
                    RULE,
                    BUILD_FILE,
                    dependency,
                    message
                )
            }
    }

    private companion object {
        const val RULE = "TechnologyDependency"
        const val BUILD_FILE = "build.gradle.kts"

        @JvmField
        val TECHNOLOGIES = setOf(
            "exposed",
            "room",
            "ktor",
            "krate",
            "sqldelight",
            "datastore",
            "retrofit",
            "okhttp",
            "cache4k",
            "memory",
            "file"
        )
    }
}
