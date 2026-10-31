package ru.astrainteractive.klibs.rules.structure.nesting.structure

import ru.astrainteractive.klibs.rules.structure.unit.api.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import java.io.File

/**
 * Reports sources in a container directory above a unit (`modules/gui/src` above `modules/gui/api`): a split
 * sub-context's directory is a container with no build file and no sources.
 */
public class ContainerSources(private val rootDirectory: File) : StructureCheck {
    override fun check(unit: BuildUnit): List<StructureViolation> {
        if (unit.isRoot) return emptyList()
        val root = rootDirectory.canonicalFile
        return generateSequence(unit.directory.canonicalFile.parentFile) { directory -> directory.parentFile }
            .takeWhile { directory -> directory != root && directory.startsWith(root) }
            .filter { directory -> File(directory, SOURCES).isDirectory }
            .map { directory ->
                val container = directory.relativeTo(root).invariantSeparatorsPath
                val message = "Move the sources of $container/$SOURCES into a unit: the container $container above " +
                    "${unit.path} holds units only."
                StructureViolation(
                    RULE,
                    "$container/$SOURCES",
                    container,
                    message
                )
            }
            .toList()
    }

    private companion object {
        const val RULE = "ContainerSources"
        const val SOURCES = "src"
    }
}
