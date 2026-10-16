package ru.astrainteractive.klibs.rules.structure.graph

import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.StructureViolation

/**
 * Checks the unit table of the cross-context rule from one unit's own dependencies: an `api` unit never depends
 * on an `impl` unit, only the entry point depends on `impl` units, and a `contract` unit depends on nothing of
 * the project. Unit kinds are read from the last segment of the project path.
 */
public class UnitDependency : StructureCheck {

    private fun findMessage(unit: BuildUnit, dependency: String): String? {
        val dependencyName = dependency.substringAfterLast(':')
        return when {
            unit.name == CONTRACT -> "A contract unit depends on nothing of the project: drop $dependency."
            dependencyName != IMPL || unit.isEntryPoint -> null
            unit.name == API -> "An api unit never depends on an impl unit: depend on the api unit next to $dependency."
            else -> "Only the entry point depends on $dependency: depend on the api unit next to it."
        }
    }

    override fun check(unit: BuildUnit): List<StructureViolation> = unit.projectDependencies
        .sorted()
        .mapNotNull { dependency ->
            val message = findMessage(unit, dependency) ?: return@mapNotNull null
            StructureViolation(RULE, BUILD_FILE, dependency, message)
        }

    private companion object {
        const val RULE = "UnitDependency"
        const val BUILD_FILE = "build.gradle.kts"
        const val API = "api"
        const val IMPL = "impl"
        const val CONTRACT = "contract"
    }
}
