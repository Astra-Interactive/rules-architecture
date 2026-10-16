package ru.astrainteractive.klibs.rules.structure.nesting

import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.StructureViolation
import java.io.File

/**
 * Reports a build unit inside the directory of another unit: when a context is split, its directory is a
 * container with no build file. The root project is not checked, since every unit sits below it.
 */
public class NestedUnit : StructureCheck {

    private fun isUnitDirectory(directory: File): Boolean =
        BUILD_FILES.any { buildFile -> File(directory, buildFile).isFile }

    private fun findNestedUnits(directory: File, depth: Int): List<File> {
        if (depth > MAX_DEPTH) return emptyList()
        val children = directory.listFiles().orEmpty()
            .filter { child -> child.isDirectory && child.name !in SKIPPED && !child.name.startsWith(".") }
        return children.flatMap { child ->
            if (isUnitDirectory(child)) listOf(child) else findNestedUnits(child, depth + 1)
        }
    }

    override fun check(unit: BuildUnit): List<StructureViolation> {
        if (unit.isRoot) return emptyList()
        return findNestedUnits(unit.directory, depth = 1)
            .map { nested -> nested.relativeTo(unit.directory).invariantSeparatorsPath }
            .sorted()
            .map { nestedPath ->
                val message = "Move the unit in $nestedPath out of ${unit.path}: a unit's directory never holds " +
                    "another unit, a split context's directory is a container with no build file."
                StructureViolation(RULE, nestedPath, nestedPath, message)
            }
    }

    private companion object {
        const val RULE = "NestedUnit"
        const val MAX_DEPTH = 3

        @JvmField
        val BUILD_FILES = listOf("build.gradle.kts", "build.gradle")

        @JvmField
        val SKIPPED = setOf("build", "src")
    }
}
