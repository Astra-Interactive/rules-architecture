@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class RootModulePlacementTest {

    @Test
    fun GIVEN_root_module_outside_the_entry_point_WHEN_checked_THEN_reports_it(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.jail.di/RootModule.kt", "class RootModule")
            .load(UnitKind.CONTEXT, rootPackage = "x.jail")

        assertEquals(
            listOf(
                "Move RootModule to the entry-point unit: the composition root lives next to the platform main " +
                    "class that creates it."
            ),
            RootModulePlacement().check(sources).map { violation -> violation.message }
        )
    }

    @Test
    fun GIVEN_root_module_outside_di_WHEN_checked_THEN_reports_it(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.bukkit/RootModule.kt", "class RootModule\n\nclass AstraRating")
            .main("x.bukkit.di/RootModule.kt", "class Other")
            .load(UnitKind.ENTRY_POINT, rootPackage = "x.bukkit")

        assertEquals(
            listOf("Move RootModule to the di package of the entry-point unit."),
            RootModulePlacement().check(sources).map { violation -> violation.message }
        )
    }

    @Test
    fun GIVEN_root_module_in_entry_point_di_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.bukkit.di/RootModule.kt", "class RootModule")
            .load(UnitKind.ENTRY_POINT, rootPackage = "x.bukkit")

        assertEquals(emptyList(), RootModulePlacement().check(sources))
    }
}
