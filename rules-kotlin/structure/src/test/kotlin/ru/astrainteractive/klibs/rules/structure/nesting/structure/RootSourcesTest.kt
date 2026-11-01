@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.nesting.structure

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import java.io.File
import kotlin.test.assertEquals

internal class RootSourcesTest {

    private fun unit(directory: File, isRoot: Boolean, isConventionPlugin: Boolean = false): BuildUnit {
        return BuildUnit(
            path = ":",
            directory = directory,
            isEntryPoint = false,
            isRoot = isRoot,
            projectDependencies = emptySet(),
            isConventionPlugin = isConventionPlugin
        )
    }

    @Test
    fun GIVEN_root_project_with_sources_WHEN_checked_THEN_reports_them(@TempDir directory: File) {
        File(directory, "src/main/kotlin").mkdirs()

        assertEquals(
            listOf("src"),
            RootSources().check(unit(directory, isRoot = true)).map(StructureViolation::path)
        )
    }

    @Test
    fun GIVEN_convention_plugin_root_with_sources_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        File(directory, "src/main/kotlin").mkdirs()

        val convention = unit(
            directory,
            isRoot = true,
            isConventionPlugin = true
        )
        assertEquals(emptyList(), RootSources().check(convention))
    }

    @Test
    fun GIVEN_root_without_sources_or_a_unit_with_sources_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        assertEquals(emptyList(), RootSources().check(unit(directory, isRoot = true)))

        File(directory, "src/main/kotlin").mkdirs()

        assertEquals(emptyList(), RootSources().check(unit(directory, isRoot = false)))
    }
}
