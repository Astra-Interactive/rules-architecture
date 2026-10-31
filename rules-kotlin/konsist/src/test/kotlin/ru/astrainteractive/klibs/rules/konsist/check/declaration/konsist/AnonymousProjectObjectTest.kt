@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class AnonymousProjectObjectTest {

    @Test
    fun GIVEN_anonymous_objects_of_project_and_library_types_WHEN_checked_THEN_reports_only_project_ones(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .dependency("x.data.dao.rating.api/RatingRepository.kt", "interface RatingRepository")
            .main("x.gui.rating.api/RatingRouter.kt", "interface RatingRouter")
            .main(
                "x.gui.rating.ktor/Bridges.kt",
                """
                internal class Bridges {
                    val router = object : RatingRouter { }
                    val repository = object: RatingRepository { }
                    val listener = object : Player.Listener { }
                    val text = "object : RatingRouter"

                    companion object : RatingRouter
                }
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.gui")

        assertEquals(
            listOf("RatingRouter", "RatingRepository"),
            AnonymousProjectObject().check(sources).map(Violation::subject)
        )
    }
}
