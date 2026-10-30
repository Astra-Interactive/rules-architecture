package ru.astrainteractive.klibs.rules.gradle.linter.task.gradle

import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.provider.ListProperty
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.TaskAction
import org.gradle.work.DisableCachingByDefault
import ru.astrainteractive.klibs.rules.gradle.linter.model.UnitKind
import ru.astrainteractive.klibs.rules.structure.graph.structure.EntryPointGraphDependency
import ru.astrainteractive.klibs.rules.structure.graph.structure.UnitDependencyCycle
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import java.io.File

@DisableCachingByDefault(because = "The graph is cheap to compute from its dependency metadata")
internal abstract class ArchitectureGraphTask : DefaultTask() {
    /**
     * One line per unit: its path, its unit kind and its project dependencies, separated by tabs.
     */
    @get:Input
    abstract val unitMetadata: ListProperty<String>

    @TaskAction
    fun check() {
        val units = unitMetadata.get().map { metadata ->
            val parts = metadata.split("\t")
            val path = parts.first()
            BuildUnit(
                path = path,
                directory = File("/"),
                isEntryPoint = parts[1] == UnitKind.ENTRY_POINT.cliName,
                isRoot = path == ":",
                projectDependencies = parts.drop(2).toSet()
            )
        }
        val violations = UnitDependencyCycle().check(units) + EntryPointGraphDependency().check(units)
        if (violations.isEmpty()) return
        val message = violations.joinToString(separator = "\n", transform = StructureViolation::message)
        throw GradleException(message)
    }
}
