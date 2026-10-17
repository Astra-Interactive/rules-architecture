@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.direction

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class KindImportDirectionTest {

    private fun SourceTree.withModelAndContracts(): SourceTree {
        return main(
            "x.gui.api.rating.model",
            "RatedPlayer.kt",
            """
            import x.gui.api.rating.api.*
            import x.gui.api.rating.util.isAllowed
            import kotlin.time.Duration

            data class RatedPlayer(val name: String, val cooldown: Duration, val other: Other)
            """
        )
            .main("x.gui.api.rating.model", "Other.kt", "class Other")
            .main(
                "x.gui.api.rating.api",
                "RatingPlayersRepository.kt",
                """
                import x.gui.api.rating.model.RatedPlayer
                import x.gui.api.rating.usecase.AddRatingUseCase

                interface RatingPlayersRepository
                """
            )
            .main(
                "x.gui.api.rating.util",
                "ConfigExt.kt",
                """
                import x.gui.api.rating.component.RatingComponent
                import x.gui.api.rating.model.RatedPlayer

                fun isAllowed(): Boolean = true
                """
            )
    }

    private fun SourceTree.withUseCasesAndImplementations(): SourceTree {
        return main(
            "x.gui.api.rating.usecase",
            "AddRatingUseCase.kt",
            """
            import x.gui.api.player.usecase.LoadPlayerUseCase
            import x.gui.api.rating.api.RatingPlayersRepository
            import x.gui.api.rating.component.RatingComponent
            import x.gui.api.rating.model.RatedPlayer
            import x.gui.api.rating.util.isAllowed

            class AddRatingUseCase
            """
        )
            .main("x.gui.api.player.usecase", "LoadPlayerUseCase.kt", "class LoadPlayerUseCase")
            .main(
                "x.gui.api.rating.memory",
                "InMemoryRatingPlayersRepository.kt",
                """
                import x.gui.api.rating.api.RatingPlayersRepository
                import x.gui.api.rating.component.RatingComponent
                import x.gui.api.rating.model.RatedPlayer

                class InMemoryRatingPlayersRepository
                """
            )
    }

    private fun SourceTree.withEntryPointsAndWiring(): SourceTree {
        return main(
            "x.gui.api.rating.component",
            "RatingComponent.kt",
            """
            import x.gui.api.di.GuiModule
            import x.gui.api.rating.Loose
            import x.gui.api.rating.api.RatingPlayersRepository
            import x.gui.api.rating.memory.InMemoryRatingPlayersRepository
            import x.gui.api.rating.slot.RatingSlot
            import x.gui.api.rating.usecase.AddRatingUseCase
            import x.gui.api.rating.util.isAllowed

            class RatingComponent
            """
        )
            .main("x.gui.api.rating.slot", "RatingSlot.kt", "class RatingSlot")
            .main(
                "x.gui.api.di",
                "GuiModule.kt",
                """
                import x.gui.api.rating.component.RatingComponent
                import x.gui.api.rating.memory.InMemoryRatingPlayersRepository

                class GuiModule
                """
            )
            .main(
                "x.gui.api.rating",
                "Loose.kt",
                """
                import x.gui.api.di.GuiModule

                class Loose
                """
            )
    }

    @Test
    fun GIVEN_unit_with_imports_against_the_direction_table_WHEN_checked_THEN_reports_each_one(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .withModelAndContracts()
            .withUseCasesAndImplementations()
            .withEntryPointsAndWiring()
            .load(UnitKind.API, unitPath = ":gui:api")

        val reported = KindImportDirection().check(sources)
            .map { violation -> "${violation.path.substringAfter("x/gui/api/")} -> ${violation.subject}" }
            .sorted()

        assertEquals(
            listOf(
                "rating/api/RatingPlayersRepository.kt -> x.gui.api.rating.usecase.AddRatingUseCase",
                "rating/component/RatingComponent.kt -> x.gui.api.di.GuiModule",
                "rating/component/RatingComponent.kt -> x.gui.api.rating.memory.InMemoryRatingPlayersRepository",
                "rating/memory/InMemoryRatingPlayersRepository.kt -> x.gui.api.rating.component.RatingComponent",
                "rating/model/RatedPlayer.kt -> x.gui.api.rating.api.*",
                "rating/model/RatedPlayer.kt -> x.gui.api.rating.util.isAllowed",
                "rating/usecase/AddRatingUseCase.kt -> x.gui.api.player.usecase.LoadPlayerUseCase",
                "rating/usecase/AddRatingUseCase.kt -> x.gui.api.rating.component.RatingComponent",
                "rating/util/ConfigExt.kt -> x.gui.api.rating.component.RatingComponent",
            ),
            reported
        )
    }

    @Test
    fun GIVEN_violation_WHEN_reported_THEN_message_names_both_places_and_the_allowed_kinds(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.jail.jail.model", "Jail.kt", "import x.jail.jail.api.JailRepository\n\nclass Jail")
            .main("x.jail.jail.api", "JailRepository.kt", "interface JailRepository")
            .load(UnitKind.CONTEXT, unitPath = ":jail")

        val violation = KindImportDirection().check(sources).single()

        assertEquals("KindImportDirection", violation.rule)
        assertEquals(
            "'jail/model' must not import x.jail.jail.api.JailRepository from 'jail/api': " +
                "a model kind depends on model only.",
            violation.message
        )
    }
}
