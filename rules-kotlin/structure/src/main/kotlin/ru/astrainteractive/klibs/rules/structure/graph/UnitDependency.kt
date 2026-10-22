package ru.astrainteractive.klibs.rules.structure.graph

import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.StructureViolation

/**
 * Checks the unit table of `arch-context.md` from one unit's own dependencies, reading unit kinds from the
 * project path: a `contract` unit depends on nothing of the project, `core` on no feature unit, and nothing but
 * the entry point depends on an `impl` unit or on a feature's platform unit (`gui/bukkit`). `core`'s own
 * platform unit (`core/bukkit`) is part of `core`.
 */
public class UnitDependency : StructureCheck {

    private fun isCore(path: String): Boolean = CORE in path.split(':')

    private fun findMessage(unit: BuildUnit, dependency: String): String? {
        val dependencyName = dependency.substringAfterLast(':')
        return when {
            unit.name == CONTRACT -> "A contract unit depends on nothing of the project: drop $dependency."
            unit.isEntryPoint -> null
            isCore(unit.path) && !isCore(dependency) -> "core depends on no feature unit: drop $dependency."
            dependencyName == IMPL && unit.name == API ->
                "An api unit never depends on an impl unit: depend on the api unit next to $dependency."
            dependencyName == IMPL -> "Only the entry point depends on $dependency: depend on the api unit next to it."
            dependencyName in PLATFORMS && !isCore(dependency) ->
                "Only the entry point depends on the platform unit $dependency: depend on the api unit next to it."
            else -> null
        }
    }

    override fun check(unit: BuildUnit): List<StructureViolation> {
        return unit.projectDependencies
            .sorted()
            .mapNotNull { dependency ->
                val message = findMessage(unit, dependency) ?: return@mapNotNull null
                StructureViolation(
                    RULE,
                    BUILD_FILE,
                    dependency,
                    message
                )
            }
    }

    private companion object {
        const val RULE = "UnitDependency"
        const val BUILD_FILE = "build.gradle.kts"
        const val API = "api"
        const val IMPL = "impl"
        const val CONTRACT = "contract"
        const val CORE = "core"

        @JvmField
        val PLATFORMS = setOf(
            "bukkit",
            "bungee",
            "fabric",
            "forge",
            "neoforge",
            "paper",
            "spigot",
            "velocity"
        )
    }
}
