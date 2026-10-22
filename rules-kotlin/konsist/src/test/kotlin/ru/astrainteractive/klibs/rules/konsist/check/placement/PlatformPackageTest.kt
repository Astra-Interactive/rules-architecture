@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class PlatformPackageTest {

    private fun writeTree(directory: File): SourceTree {
        return SourceTree(directory)
            .main("x.clipboard.android/AndroidClipboard.kt", "class AndroidClipboard")
            .main("x.clipboard.android/ClipboardExt.kt", "fun copy() = Unit")
            .main("x.clipboard.model/Clip.kt", "class Clip")
    }

    @Test
    fun GIVEN_platform_package_in_a_multiplatform_unit_WHEN_checked_THEN_reports_it_once(@TempDir directory: File) {
        val tree = writeTree(directory)
        val sources = UnitSources.load(
            kind = UnitKind.CONTEXT,
            rootPackage = "x.clipboard",
            unitPath = null,
            mainDirectories = listOf(tree.mainDirectory, File(directory, "src/androidMain/kotlin")),
            testDirectories = emptyList(),
            projectDirectory = directory
        )

        assertEquals(
            listOf(
                "Move the code of the package 'android' to the androidMain source set at the same package path: " +
                    "platform code is a source set, never a package."
            ),
            PlatformPackage().check(sources).map(Violation::message)
        )
    }

    @Test
    fun GIVEN_platform_package_in_a_single_source_set_unit_WHEN_checked_THEN_reports_nothing(
        @TempDir directory: File
    ) {
        val sources = writeTree(directory).load(UnitKind.CONTEXT, rootPackage = "x.clipboard")

        assertEquals(emptyList(), PlatformPackage().check(sources))
    }
}
