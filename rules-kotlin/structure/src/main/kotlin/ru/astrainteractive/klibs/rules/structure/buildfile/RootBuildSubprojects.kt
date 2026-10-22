package ru.astrainteractive.klibs.rules.structure.buildfile

import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.StructureViolation
import java.io.File

/**
 * Reports a `subprojects { }` or `allprojects { }` block in the build file of the root project: build logic is a
 * convention plugin applied only where a unit needs it.
 */
public class RootBuildSubprojects : StructureCheck {

    override fun check(unit: BuildUnit): List<StructureViolation> {
        if (!unit.isRoot) return emptyList()
        return BUILD_FILES
            .map { name -> File(unit.directory, name) }
            .filter { file -> file.isFile }
            .flatMap { file ->
                CROSS_PROJECT_BLOCK.findAll(file.readText())
                    .map { match -> match.groupValues[1] }
                    .distinct()
                    .map { block ->
                        val message = "Replace the $block block of the root build with a convention plugin in " +
                            "build-logic, applied only by the units that need it."
                        StructureViolation(
                            RULE,
                            file.name,
                            block,
                            message
                        )
                    }
                    .toList()
            }
    }

    private companion object {
        const val RULE = "RootBuildSubprojects"

        @JvmField
        val BUILD_FILES = listOf("build.gradle.kts", "build.gradle")

        @JvmField
        val CROSS_PROJECT_BLOCK = Regex("""\b(subprojects|allprojects)\s*\{""")
    }
}
