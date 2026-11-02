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
                val appOnlyRole = context.kind in setOf(
                    UnitKind.IMPL,
                    UnitKind.IMPL_UI,
                    UnitKind.CORE
                )
                if (!appOnlyRole) return@filter false
                if (context.kind == UnitKind.IMPL && context.publishedLibrary) return@filter false
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
                    when (unit.context?.kind) {
                        UnitKind.CORE ->
                            "Use a platform-free core role for ${unit.path}: " +
                                "mixed CORE is an app-only role."
                        UnitKind.IMPL_UI -> "Use an app product kind for ${unit.path}: IMPL_UI is an app-only role."
                        else -> "Fold ${unit.path} into its api unit or its platform unit: a plugin has no impl unit."
                    }
                )
            }
    }

    private companion object {
        const val RULE = "PluginImplUnit"
    }
}
