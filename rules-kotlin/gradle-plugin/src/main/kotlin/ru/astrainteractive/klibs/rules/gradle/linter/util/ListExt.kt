package ru.astrainteractive.klibs.rules.gradle.linter.util

import ru.astrainteractive.klibs.rules.core.unit.model.RuleId
import ru.astrainteractive.klibs.rules.structure.graph.structure.EntryPointGraphDependency
import ru.astrainteractive.klibs.rules.structure.graph.structure.PluginImplUnit
import ru.astrainteractive.klibs.rules.structure.graph.structure.UnitDependency
import ru.astrainteractive.klibs.rules.structure.graph.structure.UnitDependencyCycle
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation

internal fun List<BuildUnit>.findGraphViolations(): List<StructureViolation> {
    val disabledByPath = buildMap {
        this@findGraphViolations.forEach { unit ->
            put(unit.path, unit.context?.policy?.checks?.disabledRules.orEmpty())
        }
    }
    val root = firstOrNull(BuildUnit::isRoot)
    val rootDisabled = disabledByPath[root?.path].orEmpty()
    val cycles = UnitDependencyCycle().check(this)
        .takeUnless { RuleId.UNIT_DEPENDENCY_CYCLE.ruleName in rootDisabled }
        .orEmpty()
    val entryPointEdges = EntryPointGraphDependency().check(this)
        .filterNot { violation ->
            RuleId.ENTRY_POINT_GRAPH_DEPENDENCY.ruleName in disabledByPath[violation.path].orEmpty()
        }
    val pluginRoles = PluginImplUnit().check(this)
        .filterNot { violation ->
            RuleId.PLUGIN_IMPL_UNIT.ruleName in disabledByPath[violation.subject].orEmpty()
        }
    val directions = flatMap { unit ->
        UnitDependency().check(unit)
            .takeUnless { RuleId.UNIT_DEPENDENCY.ruleName in disabledByPath[unit.path].orEmpty() }
            .orEmpty()
    }
    return cycles + entryPointEdges + pluginRoles + directions
}
