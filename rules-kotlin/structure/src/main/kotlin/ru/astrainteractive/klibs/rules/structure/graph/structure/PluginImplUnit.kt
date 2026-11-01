package ru.astrainteractive.klibs.rules.structure.graph.structure

import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation

/**
 * Reports an `impl` unit in a declared platform plugin: a plugin splits a
 * sub-context into its `api` unit and one unit per platform, and an `impl` unit would add a unit and make nothing
 * easier.
 */
public class PluginImplUnit {
    public fun check(units: List<BuildUnit>): List<StructureViolation> {
        return units
            .filter { unit ->
                val context = unit.context ?: return@filter false
                if (context.publishedLibrary || context.kind != UnitKind.IMPL) return@filter false
                val product = requireNotNull(context.productKind) {
                    "Set linter.productKind(ProductKind.APP or ProductKind.PLATFORM_PLUGIN) " +
                        "before checking implementation unit ${unit.path}"
                }
                product == ProductKind.PLATFORM_PLUGIN
            }
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
    }
}
