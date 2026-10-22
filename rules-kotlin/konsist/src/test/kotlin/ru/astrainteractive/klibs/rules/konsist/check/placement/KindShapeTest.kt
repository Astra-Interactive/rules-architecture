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

    @Test
    fun GIVEN_database_kind_package_inside_a_technology_WHEN_checked_THEN_accepts_it(@TempDir directory: File) {
        val technology = SourceTree(File(directory, "exposed"))
            .main("x.data.exposed.rating", "ExposedRatingRepository.kt", "class ExposedRatingRepository")
            .main("x.data.exposed.database", "RatingDatabase.kt", "class RatingDatabase")
            .load(UnitKind.TECHNOLOGY, unitPath = ":modules:data:exposed")
        val context = SourceTree(File(directory, "jail"))
            .main("x.jail.jail.api", "JailRepository.kt", "interface JailRepository")
            .main("x.jail.jail.exposed.database", "JailDatabase.kt", "class JailDatabase")
            .load(UnitKind.CONTEXT, unitPath = ":jail")

        assertEquals(emptyList(), KindShape().check(technology) + KindShape().check(context))
    }

    @Test
    fun GIVEN_composable_with_one_level_of_singular_sub_packages_WHEN_checked_THEN_accepts_it(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.paywall.impl.composable.screen", "SplashScreenComposable.kt", "fun SplashScreen() = Unit")
            .main("x.paywall.impl.composable.component", "HeroCopyComposable.kt", "fun HeroCopy() = Unit")
            .main("x.paywall.impl.composable.layout", "ScrollFadeComposable.kt", "fun ScrollFade() = Unit")
            .main("x.paywall.impl.offer.composable", "OfferComposable.kt", "fun Offer() = Unit")
            .load(UnitKind.CONTEXT, unitPath = ":paywall:impl")

        assertEquals(emptyList(), KindShape().check(sources))
    }

    @Test
    fun GIVEN_composable_sub_packages_against_the_compose_rule_WHEN_checked_THEN_reports_each(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.paywall.impl.composable.screen", "SplashScreenComposable.kt", "fun SplashScreen() = Unit")
            .main("x.paywall.impl.composable", "LooseComposable.kt", "fun Loose() = Unit")
            .main("x.paywall.impl.composable.model", "BenefitContent.kt", "class BenefitContent")
            .main("x.paywall.impl.composable.util", "ModifierExt.kt", "fun format() = Unit")
            .main("x.paywall.impl.composable.theme", "PaywallMotion.kt", "object PaywallMotion")
            .main("x.paywall.impl.composable.components", "CardComposable.kt", "fun Card() = Unit")
            .main("x.paywall.impl.composable.address", "AddressComposable.kt", "fun Address() = Unit")
            .main("x.paywall.impl.composable.layout.frame", "FrameComposable.kt", "fun Frame() = Unit")
            .load(UnitKind.CONTEXT, unitPath = ":paywall:impl")

        val messages = KindShape().check(sources).map { violation -> violation.message }.sorted()

        assertEquals(
            listOf(
                "Move BenefitContent out of 'composable/model': model is a kind of the level, never a sub-package " +
                    "of composable.",
                "Move FrameComposable out of 'composable/layout/frame': a composable sub-package holds only files.",
                "Move LooseComposable into a sub-package of 'composable': composable holds files or sub-packages, " +
                    "never both.",
                "Move ModifierExt out of 'composable/util': util is a kind of the level, never a sub-package of " +
                    "composable.",
                "Move PaywallMotion out of 'composable/theme': theme is a kind of the level, never a sub-package of " +
                    "composable.",
                "Rename 'composable/components' to the singular: a composable sub-package is named like a kind.",
            ),
            messages
        )
    }
}
