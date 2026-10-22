package ru.astrainteractive.klibs.rules.structure.graph

import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.StructureViolation

/**
 * Reports a dependency of an `api` unit on anything but another `api` unit, `core` or a storage contracts unit
 * (`…:dao`, `…:contract`), judged by path. A dependency on an `impl` unit is left to [UnitDependency].
 */
public class ApiUnitDependency : StructureCheck {

    private fun isAllowed(dependency: String): Boolean {
        val segments = dependency.split(':')
        return segments.last() in ALLOWED_NAMES || CORE in segments
    }

    override fun check(unit: BuildUnit): List<StructureViolation> {
        if (unit.name != API) return emptyList()
        return unit.projectDependencies
            .sorted()
            .filterNot { dependency -> isAllowed(dependency) || dependency.substringAfterLast(':') == IMPL }
            .map { dependency ->
                val message = "An api unit depends only on other api units, core and the storage contracts unit: " +
                    "drop $dependency or depend on its api unit."
                StructureViolation(
                    RULE,
                    BUILD_FILE,
                    dependency,
                    message
                )
            }
    }

    private companion object {
        const val RULE = "ApiUnitDependency"
        const val BUILD_FILE = "build.gradle.kts"
        const val API = "api"
        const val IMPL = "impl"
        const val CORE = "core"

        @JvmField
        val ALLOWED_NAMES = setOf(
            API,
            "dao",
            "contract"
        )
    }
}
