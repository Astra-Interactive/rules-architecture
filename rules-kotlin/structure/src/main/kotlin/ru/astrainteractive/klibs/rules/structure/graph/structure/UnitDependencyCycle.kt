package ru.astrainteractive.klibs.rules.structure.graph.structure

import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation

/**
 * Reports cycles in the complete project-unit graph. Dependencies on paths absent from [check]'s input are left to
 * Gradle, since they cannot form a cycle in the described graph.
 */
public class UnitDependencyCycle {
    private fun canonicalCycle(cycle: List<String>): String {
        val first = cycle.indices.minBy(cycle::get)
        val ordered = cycle.drop(first).plus(cycle.take(first))
        return ordered.plus(ordered.first()).joinToString(" -> ")
    }

    private fun cyclesFrom(
        current: String,
        dependencies: Map<String, Set<String>>,
        path: List<String>
    ): List<String> {
        return dependencies[current].orEmpty().flatMap { dependency ->
            if (dependency in path) {
                val cycle = path.drop(path.indexOf(dependency))
                listOf(canonicalCycle(cycle))
            } else {
                cyclesFrom(
                    dependency,
                    dependencies,
                    path.plus(dependency)
                )
            }
        }
    }

    public fun check(units: List<BuildUnit>): List<StructureViolation> {
        val dependencies = buildMap {
            units.forEach { unit -> put(unit.path, unit.projectDependencies) }
        }
        return dependencies.keys
            .flatMap { path ->
                cyclesFrom(
                    path,
                    dependencies,
                    listOf(path)
                )
            }
            .toSet()
            .sorted()
            .map { cycle ->
                StructureViolation(
                    RULE,
                    ".",
                    cycle,
                    "Break the unit dependency cycle $cycle: the project units form an acyclic graph."
                )
            }
    }

    private companion object {
        const val RULE = "UnitDependencyCycle"
    }
}
