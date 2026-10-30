package ru.astrainteractive.klibs.rules.structure.graph.structure

import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation

/**
 * Reports a dependency on an entry-point unit across the whole graph, where each unit knows whether it is an
 * entry point (an `application` plugin or an `instances` path), not only by the dependency's path.
 */
public class EntryPointGraphDependency {
    public fun check(units: List<BuildUnit>): List<StructureViolation> {
        val entryPoints = units.filter(BuildUnit::isEntryPoint).map(BuildUnit::path).toSet()
        return units.flatMap { unit ->
            unit.projectDependencies
                .filter { dependency -> dependency in entryPoints }
                .sorted()
                .map { dependency ->
                    StructureViolation(
                        RULE,
                        unit.path,
                        dependency,
                        "Drop the dependency of ${unit.path} on the entry point $dependency: nothing depends on the " +
                            "entry point; depend on the api unit that holds what you need."
                    )
                }
        }
    }

    private companion object {
        const val RULE = "EntryPointGraphDependency"
    }
}
