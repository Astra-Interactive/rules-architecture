@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.nesting

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.structure.unit.BuildUnit
import java.io.File
import kotlin.test.assertEquals

internal class RootSourcesTest {

    private fun unit(directory: File, isRoot: Boolean): BuildUnit = BuildUnit(
        path = ":",
        directory = directory,
        isEntryPoint = false,
        isRoot = isRoot,
        projectDependencies = emptySet()
    )

    @Test
    fun GIVEN_root_project_with_sources_WHEN_checked_THEN_reports_them(@TempDir directory: File) {
        File(directory, "src/main/kotlin").mkdirs()

        assertEquals(
            listOf("src"),
            RootSources().check(unit(directory, isRoot = true)).map { violation -> violation.path }
        )
    }

    @Test
    fun GIVEN_root_without_sources_or_a_unit_with_sources_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        assertEquals(emptyList(), RootSources().check(unit(directory, isRoot = true)))

        File(directory, "src/main/kotlin").mkdirs()

        assertEquals(emptyList(), RootSources().check(unit(directory, isRoot = false)))
    }
}
