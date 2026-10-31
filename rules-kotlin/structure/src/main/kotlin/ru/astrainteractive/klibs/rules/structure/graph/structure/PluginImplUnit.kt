package ru.astrainteractive.klibs.rules.structure.graph.structure

import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import ru.astrainteractive.klibs.rules.structure.unit.structure.isPlatformUnitName

/**
 * Reports an `impl` unit in a plugin, a build with a platform unit (`bukkit`, `paper`, …): a plugin splits a
 * sub-context into its `api` unit and one unit per platform, and an `impl` unit would add a unit and make nothing
 * easier.
 */
public class PluginImplUnit {
    public fun check(units: List<BuildUnit>): List<StructureViolation> {
        val isPlugin = units.any { unit -> isPlatformUnitName(unit.name) }
        if (!isPlugin) return emptyList()
        return units
            .filter { unit -> unit.name == IMPL }
            .map { unit ->
                StructureViolation(
                    RULE,
                    ".",
                    unit.path,
                    "Fold ${unit.path} into its api unit or its platform unit: a plugin has no impl unit."
                )
            }
    }

    private companion object {
        const val RULE = "PluginImplUnit"
        const val IMPL = "impl"
    }
}
