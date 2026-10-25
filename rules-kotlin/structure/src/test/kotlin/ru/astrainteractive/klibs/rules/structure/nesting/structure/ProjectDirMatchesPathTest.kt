@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.nesting.structure

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import java.io.File
import kotlin.test.assertEquals

internal class ProjectDirMatchesPathTest {

    private fun unit(path: String, directory: File, isRoot: Boolean): BuildUnit {
        return BuildUnit(
            path = path,
            directory = directory,
            isEntryPoint = false,
            isRoot = isRoot,
            projectDependencies = emptySet()
        )
    }

    @Test
    fun GIVEN_unit_in_another_directory_WHEN_checked_THEN_reports_it(@TempDir root: File) {
        val unit = unit(
            ":jail:api",
            File(root, "modules/jail-api"),
            isRoot = false
        )

        assertEquals(
            listOf(
                "Move :jail:api to jail/api or rename it after modules/jail-api: a unit's name equals its location, " +
                    "never a projectDir override."
            ),
            ProjectDirMatchesPath(root).check(unit).map(StructureViolation::message)
        )
    }

    @Test
    fun GIVEN_unit_at_its_path_or_the_root_WHEN_checked_THEN_reports_nothing(@TempDir root: File) {
        val module = unit(
            ":modules:jail:api",
            File(root, "modules/jail/api"),
            isRoot = false
        )
        val rootUnit = unit(
            ":",
            root,
            isRoot = true
        )

        assertEquals(
            emptyList(),
            ProjectDirMatchesPath(root).check(module) + ProjectDirMatchesPath(root).check(rootUnit)
        )
    }
}
