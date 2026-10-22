@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
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
}
