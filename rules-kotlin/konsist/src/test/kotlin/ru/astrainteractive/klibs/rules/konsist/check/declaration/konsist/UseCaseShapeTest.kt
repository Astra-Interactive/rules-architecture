@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
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
    fun GIVEN_pass_through_use_cases_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.jail.api.jail.usecase",
                "FindJailUseCase.kt",
                """
                class FindJailUseCase(private val jailRepository: JailRepository) {
                    suspend operator fun invoke(id: JailId): Jail? = jailRepository.findJail(id)
                }
                """
            )
            .main(
                "x.jail.api.jail.usecase",
                "SaveInmateUseCase.kt",
                """
                class SaveInmateUseCase(private val jailRepository: JailRepository) {
                    suspend operator fun invoke(inmate: JailInmate) {
                        return this.jailRepository.save(inmate)
                    }
                }
                """
            )
            .load(UnitKind.API, unitPath = ":jail:api")

        assertEquals(
            listOf(
                "FindJailUseCase only passes the call on to jailRepository.findJail: make that call from the entry " +
                    "point; a use case exists for a flow of more than one step.",
                "SaveInmateUseCase only passes the call on to jailRepository.save: make that call from the entry " +
                    "point; a use case exists for a flow of more than one step."
            ),
            UseCaseShape().check(sources).map(Violation::message).sorted()
        )
    }

    @Test
    fun GIVEN_use_cases_of_several_steps_or_other_calls_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.jail.api.jail.usecase",
                "FreeInmateUseCase.kt",
                """
                class FreeInmateUseCase(private val jailRepository: JailRepository, private val clock: Clock) {
                    suspend operator fun invoke(inmate: JailInmate) {
                        if (!inmate.isSentenceOver(clock.now())) return
                        jailRepository.free(inmate)
                    }
                }
                """
            )
            .main(
                "x.jail.api.jail.usecase",
                "RenameJailUseCase.kt",
                """
                class RenameJailUseCase(jailRepository: JailRepository) {
                    operator fun invoke(name: String) = helper.rename(name)
                }
                """
            )
            .main(
                "x.jail.api.jail.usecase",
                "CountJailsUseCase.kt",
                """
                class CountJailsUseCase(private val jailRepository: JailRepository) {
                    suspend operator fun invoke() = jailRepository.loadJails(Filter.all()).size
                }
                """
            )
            .load(UnitKind.API, unitPath = ":jail:api")

        assertEquals(emptyList(), UseCaseShape().check(sources))
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
