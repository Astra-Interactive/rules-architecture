package ru.astrainteractive.klibs.rules.gradle.linter.task.gradle

import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
import org.gradle.api.provider.ListProperty
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.TaskAction
import org.gradle.work.DisableCachingByDefault
import ru.astrainteractive.klibs.rules.structure.graph.structure.UnitDependencyCycle
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import java.io.File

@DisableCachingByDefault(because = "The graph is cheap to compute from its dependency metadata")
internal abstract class ArchitectureGraphTask : DefaultTask() {
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
                isEntryPoint = path.startsWith(ENTRY_POINT_PATH_PREFIX),
                isRoot = path == ":",
                projectDependencies = parts.drop(1).toSet()
            )
        }
        val cycles = UnitDependencyCycle().check(units)
        if (cycles.isEmpty()) return
        val message = cycles.joinToString(separator = "\n", transform = StructureViolation::message)
        throw GradleException(message)
    }

    private companion object {
        const val ENTRY_POINT_PATH_PREFIX = ":instances:"
    }
}
