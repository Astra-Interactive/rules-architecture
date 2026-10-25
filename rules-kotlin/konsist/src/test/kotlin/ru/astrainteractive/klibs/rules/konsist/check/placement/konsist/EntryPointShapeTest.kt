@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class EntryPointShapeTest {

    @Test
    fun GIVEN_entry_point_with_feature_code_WHEN_checked_THEN_reports_each_file(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.bukkit/AstraRating.kt", "class AstraRating")
            .main("x.bukkit.di/RootModule.kt", "class RootModule")
            .main("x.bukkit.rating.command/RatingCommand.kt", "class RatingCommand")
            .main("x.bukkit.util/LocationExt.kt", "fun format() = Unit")
            .main("y.other/Outside.kt", "class Outside")
            .load(UnitKind.ENTRY_POINT, rootPackage = "x.bukkit")

        assertEquals(
            listOf(
                "LocationExt",
                "Outside",
                "RatingCommand"
            ),
            EntryPointShape().check(sources).map(Violation::subject).sorted()
        )
    }
}
