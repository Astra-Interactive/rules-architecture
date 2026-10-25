@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.nesting.structure

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import java.io.File
import kotlin.test.assertEquals

internal class NestedUnitTest {

    private fun touch(directory: File, path: String) {
        val file = File(directory, path)
        file.parentFile.mkdirs()
        file.writeText("")
    }

    private fun unit(directory: File, isRoot: Boolean): BuildUnit {
        return BuildUnit(
            path = ":dictionary",
            directory = directory,
            isEntryPoint = false,
            isRoot = isRoot,
            projectDependencies = emptySet()
        )
    }

    @Test
    fun GIVEN_units_inside_the_unit_directory_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        touch(directory, "build.gradle.kts")
        touch(directory, "contract/build.gradle.kts")
        touch(directory, "contract/inner/build.gradle.kts")
        touch(directory, "legacy/groovy/build.gradle")
        touch(directory, "src/main/resources/sample/build.gradle.kts")
        touch(directory, "build/tmp/build.gradle.kts")
        touch(directory, ".gradle/cache/build.gradle.kts")
        touch(directory, "a/b/c/deep/build.gradle.kts")
        touch(directory, "docs/README.md")

        val subjects = NestedUnit().check(unit(directory, isRoot = false)).map(StructureViolation::subject)

        assertEquals(listOf("contract", "legacy/groovy"), subjects)
    }

    @Test
    fun GIVEN_root_project_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        touch(directory, "jail/build.gradle.kts")

        assertEquals(emptyList(), NestedUnit().check(unit(directory, isRoot = true)))
    }
}
