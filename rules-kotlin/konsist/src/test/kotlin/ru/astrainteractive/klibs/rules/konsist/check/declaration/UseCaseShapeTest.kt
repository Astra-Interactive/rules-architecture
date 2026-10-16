@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class UseCaseShapeTest {

    @Test
    fun GIVEN_use_cases_with_extra_entries_or_called_use_cases_WHEN_checked_THEN_reports_each(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "com.example.jail",
                "JailPlayer.kt",
                """
                class JailPlayer {
                    operator fun invoke() = Unit
                    fun undo() = Unit
                }
                """
            )
            .main(
                "com.example.jail",
                "FreeInmate.kt",
                """
                class FreeInmate(private val jailPlayer: JailPlayer?, private val count: Int) {
                    private fun log() = Unit
                    operator fun invoke() = log()
                }
                """
            )
            .main("com.example.jail", "JailApi.kt", "interface JailApi")
            .main("com.example.jail.model", "Jail.kt", "class Jail")
            .load(UnitKind.CONTEXT)

        val reported = UseCaseShape().check(sources).map { violation -> violation.subject }

        assertEquals(listOf("FreeInmate -> JailPlayer", "JailPlayer"), reported.sorted())
    }

    @Test
    fun GIVEN_use_case_with_one_entry_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "com.example.jail",
                "JailPlayer.kt",
                "class JailPlayer(private val other: JailPlayer) { operator fun invoke() = Unit }"
            )
            .load(UnitKind.CONTEXT)

        assertEquals(emptyList(), UseCaseShape().check(sources))
    }
}
