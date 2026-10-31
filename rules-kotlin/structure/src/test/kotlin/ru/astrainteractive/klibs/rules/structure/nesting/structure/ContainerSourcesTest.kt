@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.nesting.structure

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import java.io.File
import kotlin.test.assertEquals

internal class ContainerSourcesTest {
    private fun unit(directory: File, path: String): BuildUnit {
        return BuildUnit(
            path = path,
            directory = directory,
            isEntryPoint = false,
            isRoot = false,
            projectDependencies = emptySet()
        )
    }

    @Test
    fun GIVEN_sources_in_the_container_of_a_unit_WHEN_checked_THEN_reports_them(@TempDir root: File) {
        val unitDirectory = File(root, "modules/gui/api").apply { mkdirs() }
        File(root, "modules/gui/src/main/kotlin").mkdirs()
        File(root, "src/main/kotlin").mkdirs()

        val violations = ContainerSources(root).check(unit(unitDirectory, ":modules:gui:api"))

        assertEquals(listOf("modules/gui/src"), violations.map(StructureViolation::path))
    }

    @Test
    fun GIVEN_containers_without_sources_WHEN_checked_THEN_reports_nothing(@TempDir root: File) {
        val unitDirectory = File(root, "modules/gui/api").apply { mkdirs() }
        File(unitDirectory, "src/main/kotlin").mkdirs()

        assertEquals(emptyList(), ContainerSources(root).check(unit(unitDirectory, ":modules:gui:api")))
    }
}
