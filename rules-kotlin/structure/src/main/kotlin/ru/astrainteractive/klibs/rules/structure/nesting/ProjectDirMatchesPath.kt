package ru.astrainteractive.klibs.rules.structure.nesting

import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.StructureViolation
import java.io.File

/**
 * Reports a unit whose directory differs from its Gradle path read as a directory below [rootDirectory]
 * (`:modules:jail:api` → `modules/jail/api`): a unit's logical name equals its physical location.
 */
public class ProjectDirMatchesPath(private val rootDirectory: File) : StructureCheck {

    override fun check(unit: BuildUnit): List<StructureViolation> {
        if (unit.isRoot) return emptyList()
        val expectedPath = unit.path.removePrefix(":").replace(':', '/')
        val expected = File(rootDirectory, expectedPath)
        if (expected.canonicalFile == unit.directory.canonicalFile) return emptyList()
        val actual = unit.directory.canonicalFile.relativeToOrSelf(rootDirectory.canonicalFile)
        val actualPath = actual.invariantSeparatorsPath
        val message = "Move ${unit.path} to $expectedPath or rename it after $actualPath: a unit's name equals its " +
            "location, never a projectDir override."
        return listOf(
            StructureViolation(
                RULE,
                ".",
                unit.path,
                message
            )
        )
    }

    private companion object {
        const val RULE = "ProjectDirMatchesPath"
    }
}
