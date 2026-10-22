@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class UseCaseShapeTest {

    @Test
    fun GIVEN_use_cases_out_of_shape_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.gui.api.rating.usecase",
                "AddRatingUseCase.kt",
                """
                class AddRatingUseCase(private val loadPlayer: LoadPlayerUseCase?, private val count: Int) {
                    operator fun invoke() = Unit
                    fun undo() = Unit
                }
                """
            )
            .main(
                "x.gui.api.rating.usecase",
                "RatingCalculator.kt",
                "class RatingCalculator"
            )
            .main(
                "x.gui.api.rating.component",
                "LoadPlayerUseCase.kt",
                "class LoadPlayerUseCase {\n    fun invoke() = Unit\n}"
            )
            .main(
                "x.gui.api.rating.model",
                "Rating.kt",
                "class Rating"
            )
            .load(UnitKind.API, unitPath = ":gui:api")

        val messages = UseCaseShape().check(sources).map(Violation::message).sorted()

        assertEquals(
            listOf(
                "AddRatingUseCase has the public functions invoke, undo: $ONE_ENTRY",
                "AddRatingUseCase takes LoadPlayerUseCase: a use case never calls another use case; " +
                    "the entry point calls both.",
                "LoadPlayerUseCase has the public functions invoke: $ONE_ENTRY",
                "Move LoadPlayerUseCase to the usecase package of its concept.",
                "Name RatingCalculator <Verb><Object>UseCase, or move it out of the usecase package.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_use_case_in_shape_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.gui.api.rating.usecase",
                "AddRatingUseCase.kt",
                """
                class AddRatingUseCase {
                    private fun log() = Unit
                    operator fun invoke() = log()
                }
                """
            )
            .main(
                "x.gui.api.rating.usecase",
                "EmptyUseCase.kt",
                "class EmptyUseCase"
            )
            .load(UnitKind.API, unitPath = ":gui:api")

        assertEquals(
            listOf("EmptyUseCase has the public functions none: $ONE_ENTRY"),
            UseCaseShape().check(sources).map(Violation::message)
        )
    }

    private companion object {
        const val ONE_ENTRY = "a use case has one public entry, operator fun invoke."
    }
}
