@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class FieldDependencyConstructionTest {

    @Test
    fun GIVEN_fields_constructing_dependencies_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.rating.api/RatingRepository.kt", "class ExposedRatingRepository(val database: Database)")
            .main("x.rating.model/Rating.kt", "class Rating")
            .main(
                "x.rating.command/RatingCommand.kt",
                """
                class RatingCommand {
                    private val repository = ExposedRatingRepository(Database())
                    private val lazyRepository by lazy { ExposedRatingRepository(Database()) }
                    private val rating = Rating()
                    private val name = "rating"
                    private val external = Json(true)
                }
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x")

        assertEquals(
            listOf(
                "RatingCommand constructs ExposedRatingRepository in a field: construct it in the DI module and pass " +
                    "it in through the constructor.",
                "RatingCommand constructs ExposedRatingRepository in a field: construct it in the DI module and pass " +
                    "it in through the constructor.",
            ),
            FieldDependencyConstruction().check(sources).map { violation -> violation.message }
        )
    }

    @Test
    fun GIVEN_di_module_constructing_dependencies_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.rating.api/RatingRepository.kt", "class ExposedRatingRepository(val database: Database)")
            .main(
                "x.rating.di/RatingModule.kt",
                "class RatingModule {\n    val repository = ExposedRatingRepository(Database())\n}"
            )
            .load(UnitKind.CONTEXT, rootPackage = "x")

        assertEquals(emptyList(), FieldDependencyConstruction().check(sources))
    }
}
