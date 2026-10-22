@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.buildfile

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.StructureViolation
import java.io.File
import kotlin.test.assertEquals

internal class RootBuildSubprojectsTest {

    private fun unit(directory: File, isRoot: Boolean): BuildUnit {
        return BuildUnit(
            path = ":",
            directory = directory,
            isEntryPoint = false,
            isRoot = isRoot,
            projectDependencies = emptySet()
        )
    }

    @Test
    fun GIVEN_root_build_with_cross_project_blocks_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val script = """
            subprojects {
            }
            allprojects{ }
            subprojects { }
        """.trimIndent()
        File(directory, "build.gradle.kts").writeText(script)

        val violations = RootBuildSubprojects().check(unit(directory, isRoot = true))

        assertEquals(listOf("subprojects", "allprojects"), violations.map(StructureViolation::subject))
        assertEquals(
            "Replace the subprojects block of the root build with a convention plugin in build-logic, applied " +
                "only by the units that need it.",
            violations.first().message
        )
    }

    @Test
    fun GIVEN_clean_root_or_a_unit_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        File(directory, "build.gradle").writeText("plugins { } // subprojectsCount")
        val root = RootBuildSubprojects().check(unit(directory, isRoot = true))

        File(directory, "build.gradle.kts").writeText("subprojects { }")
        val module = RootBuildSubprojects().check(unit(directory, isRoot = false))

        assertEquals(emptyList(), root + module)
    }
}
