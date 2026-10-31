@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals
import kotlin.test.assertTrue

internal class FakePlacementTest {

    @Test
    fun GIVEN_fakes_in_tests_WHEN_checked_THEN_reports_those_outside_a_fake_package(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .test(
                "com.example.jail.data.fake",
                "FakeJailRepository.kt",
                "class FakeJailRepository"
            )
            .test(
                "com.example.jail.data",
                "FakeBankRepository.kt",
                "class FakeBankRepository"
            )
            .test(
                "com.example.jail",
                "FakeClock.kt",
                "object FakeClock\n\nclass Faker"
            )
            .load(UnitKind.LIBRARY)

        val subjects = FakePlacement().check(sources).map(Violation::subject)

        assertEquals(listOf("FakeBankRepository", "FakeClock"), subjects.sorted())
    }

    @Test
    fun GIVEN_fakes_in_main_sources_WHEN_checked_THEN_reports_each_with_the_test_tree_as_destination(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "com.example.jail.jail.api",
                "JailRepository.kt",
                "interface JailRepository"
            )
            .main(
                "com.example.jail.jail.fake",
                "FakeJailRepository.kt",
                "class FakeJailRepository"
            )
            .main(
                "com.example.jail.jail.memory",
                "FakeClock.kt",
                "object FakeClock\n\nclass Faker"
            )
            .load(UnitKind.CONTEXT, unitPath = ":jail")

        val violations = FakePlacement().check(sources)

        assertEquals(
            listOf("FakeClock", "FakeJailRepository"),
            violations.map(Violation::subject).sorted()
        )
        assertTrue(violations.all { violation -> violation.message.contains("or in testFixtures") })
    }

    @Test
    fun GIVEN_fake_unit_of_a_multiplatform_feature_WHEN_checked_THEN_accepts_its_main_fakes(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "com.example.monetization.fake.offer",
                "FakeOfferRepository.kt",
                "class FakeOfferRepository"
            )
            .main(
                "com.example.monetization.fake.plan",
                "FakePlanRepository.kt",
                "class FakePlanRepository"
            )
            .load(UnitKind.CONTEXT, unitPath = ":monetization:fake")

        assertEquals(emptyList(), FakePlacement().check(sources))
    }

    @Test
    fun GIVEN_product_code_in_a_fake_unit_WHEN_checked_THEN_reports_it(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("com.example.monetization.fake.offer/FakeOfferRepository.kt", "class FakeOfferRepository")
            .main("com.example.monetization.fake.offer/OfferPricing.kt", "class OfferPricing")
            .load(UnitKind.CONTEXT, unitPath = ":monetization:fake")

        assertEquals(listOf("OfferPricing"), FakePlacement().check(sources).map(Violation::subject))
    }

    @Test
    fun GIVEN_main_class_and_test_class_named_alike_WHEN_checked_THEN_reports_only_the_main_class(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.rating.command",
                "FakePlacement.kt",
                "class FakePlacement : ArchitectureCheck"
            )
            .test(
                "x.rating.command",
                "FakePlacementTest.kt",
                "class FakePlacementTest"
            )
            .load(UnitKind.CONTEXT, rootPackage = "x")

        assertEquals(listOf("FakePlacement"), FakePlacement().check(sources).map(Violation::subject))
    }

    @Test
    fun GIVEN_test_fake_of_another_units_contract_WHEN_checked_THEN_reports_it(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .dependency("x.data.dao.rating.api/RatingRepository.kt", "interface RatingRepository")
            .main("x.gui.rating.api/RatingGuiRouter.kt", "interface RatingGuiRouter")
            .test("x.gui.rating.fake/FakeRatingRepository.kt", "class FakeRatingRepository : RatingRepository")
            .test("x.gui.rating.fake/FakeRatingGuiRouter.kt", "class FakeRatingGuiRouter : RatingGuiRouter")
            .load(UnitKind.API, rootPackage = "x.gui")

        val violation = FakePlacement().check(sources).single()

        assertEquals("FakeRatingRepository", violation.subject)
        assertEquals(
            "Move FakeRatingRepository to testFixtures of the unit that owns RatingRepository: a fake that tests of " +
                "other units need is shared from there, never copied into each one.",
            violation.message
        )
    }

    @Test
    fun GIVEN_fake_of_another_units_contract_in_test_fixtures_WHEN_checked_THEN_reports_nothing(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .dependency("x.data.dao.rating.api/RatingRepository.kt", "interface RatingRepository")
            .main(
                "main",
                "x.gui.rating.api",
                "RatingGuiRouter.kt",
                "interface RatingGuiRouter"
            )
            .test(
                "testFixtures",
                "x.gui.rating.fake",
                "FakeRatingRepository.kt",
                "class FakeRatingRepository : RatingRepository"
            )
            .loadSourceSets(
                UnitKind.API,
                mainSourceSet = "main",
                testSourceSet = "testFixtures"
            )

        assertEquals(emptyList(), FakePlacement().check(sources).map(Violation::subject))
    }
}
