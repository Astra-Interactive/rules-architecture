@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class DiPlacementTest {

    @Test
    fun GIVEN_di_modules_WHEN_checked_THEN_reports_those_outside_a_di_package(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.gui.api.di",
                "GuiModule.kt",
                "class GuiModule"
            )
            .main(
                "x.gui.api.rating.players.di",
                "RatingPlayersModule.kt",
                "class RatingPlayersModule\n\nobject PlayersWiringModule"
            )
            .main(
                "x.gui.api.rating.model",
                "StorageModule.kt",
                "class StorageModule"
            )
            .main(
                "x.gui.api.rating.component",
                "RatingComponent.kt",
                "object ComponentModule"
            )
            .load(UnitKind.API, unitPath = ":gui:api")

        val subjects = DiPlacement().check(sources).map(Violation::subject).sorted()

        assertEquals(listOf("ComponentModule", "StorageModule"), subjects)
    }
}
