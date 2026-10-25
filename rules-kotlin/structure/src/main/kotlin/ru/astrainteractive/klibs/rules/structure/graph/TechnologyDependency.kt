package ru.astrainteractive.klibs.rules.structure.graph

import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.StructureViolation

/**
 * Reports a technology dependency outside the unit direction table: storage is reached through contracts, and an
 * unrelated technology unit may not implement another technology's unit.
 */
public class TechnologyDependency : StructureCheck {

    private fun isTechnology(path: String): Boolean = path.substringAfterLast(':') in TECHNOLOGIES

    override fun check(unit: BuildUnit): List<StructureViolation> {
        return UnitDependencyDirection.findForbidden(unit)
            .filter(::isTechnology)
            .map { dependency ->
                val message = "Depend on storage contracts and platform-free core instead of the technology unit " +
                    "$dependency: only compatible technology units and the entry point may depend on it."
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
