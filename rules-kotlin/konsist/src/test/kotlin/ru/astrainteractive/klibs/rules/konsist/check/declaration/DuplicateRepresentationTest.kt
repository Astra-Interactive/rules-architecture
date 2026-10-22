@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class DuplicateRepresentationTest {

    @Test
    fun GIVEN_technology_copy_of_a_model_WHEN_checked_THEN_reports_it(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.data.exposed.model/Rating.kt", "data class Rating(val id: Long, val value: Int)")
            .main("x.data.exposed.rating/RatingRow.kt", "data class RatingRow(val value: Int, val id: Long)")
            .load(UnitKind.TECHNOLOGY, rootPackage = "x.data.exposed")

        assertEquals(
            listOf(
                "RatingRow copies the fields of Rating: one representation per thing; keep a technology shape only " +
                    "when the technology dictates it, and map at its edge."
            ),
            DuplicateRepresentation().check(sources).map { violation -> violation.message }
        )
    }

    @Test
    fun GIVEN_technology_shapes_of_their_own_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.jail.model/Jail.kt",
                """
                data class Jail(val id: Long, val name: String)

                data class Cell(val id: Long)
                """
            )
            .main("x.jail.file/JailFile.kt", "data class JailFile(val id: Long, val title: String)")
            .main(
                "x.jail.memory/CellEntry.kt",
                """
                data class CellEntry(val id: Long)

                class Store(val id: Long, val name: String)
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x")

        assertEquals(emptyList(), DuplicateRepresentation().check(sources))
    }
}
