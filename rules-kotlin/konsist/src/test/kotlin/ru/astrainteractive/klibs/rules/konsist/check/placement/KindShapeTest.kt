@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class KindShapeTest {

    @Test
    fun GIVEN_loose_files_and_nested_kind_packages_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.gui.api", "GuiApi.kt", "class GuiApi")
            .main("x.gui.api.rating", "RatingHelpers.kt", "fun rate() = Unit")
            .main("x.gui.api.rating.model", "Rating.kt", "class Rating")
            .main("x.gui.api.rating.model.extra", "Extra.kt", "class Extra")
            .main("x.gui.api.rating.memory.cache", "Cache.kt", "class Cache")
            .main("x.gui.api.di", "GuiModule.kt", "class GuiModule")
            .load(UnitKind.API, unitPath = ":gui:api")

        val messages = KindShape().check(sources).map { violation -> violation.message }.sorted()

        assertEquals(
            listOf(
                "Move Extra out of 'rating/model/extra': a kind package holds only files.",
                "Move GuiApi into a kind package: the unit root holds only packages.",
                "Move RatingHelpers into a kind package of 'rating': a concept package holds only packages.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_technology_unit_and_entry_point_WHEN_checked_THEN_accepts_their_exceptions(@TempDir directory: File) {
        val technology = SourceTree(File(directory, "exposed"))
            .main("x.data.exposed.rating", "ExposedRatingRepository.kt", "class ExposedRatingRepository")
            .main("x.data.exposed.rating.table", "UserRatingTable.kt", "object UserRatingTable")
            .load(UnitKind.TECHNOLOGY, unitPath = ":modules:data:exposed")
        val entryPoint = SourceTree(File(directory, "bukkit"))
            .main("x.astrarating", "AstraRating.kt", "class AstraRating")
            .main("x.astrarating.di", "RootModule.kt", "class RootModule")
            .load(UnitKind.ENTRY_POINT, unitPath = ":instances:bukkit")

        assertEquals(emptyList(), KindShape().check(technology) + KindShape().check(entryPoint))
    }

    @Test
    fun GIVEN_file_in_a_compose_theme_package_WHEN_checked_THEN_accepts_it_as_a_kind(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.paywall.impl.splash.theme", "PaywallMotion.kt", "internal object PaywallMotion")
            .main("x.paywall.impl.splash.model", "Plan.kt", "class Plan")
            .load(UnitKind.CONTEXT, unitPath = ":paywall:impl")

        assertEquals(emptyList(), KindShape().check(sources))
    }
}
