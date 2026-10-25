@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class UtilFileNameTest {

    @Test
    fun GIVEN_misnamed_util_files_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("com.example.jail.util/Inmates.kt", "fun JailInmate.isSentenceOver(): Boolean = true")
            .main("com.example.jail.model/Utils.kt", "fun format(): String = \"\"")
            .load(UnitKind.CONTEXT, rootPackage = "com.example")

        val messages = UtilFileName().check(sources).map(Violation::message).sorted()

        assertEquals(
            listOf(
                "Rename Inmates.kt to JailInmateExt.kt: a util file of JailInmate extensions is named after its " +
                    "receiver.",
                "Rename Utils.kt after what its functions do, or <Subject>Ext.kt for extensions.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_util_files_named_after_their_subject_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("com.example.jail.util/JailInmateExt.kt", "fun JailInmate.isSentenceOver(): Boolean = true")
            .main(
                "com.example.jail.util/TransferLimit.kt",
                """
                fun isWithinLimit(limit: Int): Boolean = true

                fun Transfer.isLarge(): Boolean = true
                """
            )
            .main(
                "com.example.jail.util/DurationFormat.kt",
                """
                fun Duration.format(): String = ""

                fun Instant.format(): String = ""
                """
            )
            .main("com.example.jail.util/Constants.kt", "val LIMIT = 1")
            .main("com.example.jail.model/Inmates.kt", "fun JailInmate.isSentenceOver(): Boolean = true")
            .load(UnitKind.CONTEXT, rootPackage = "com.example")

        assertEquals(emptyList(), UtilFileName().check(sources))
    }
}
