@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class FakePlacementTest {

    @Test
    fun GIVEN_fakes_in_tests_WHEN_checked_THEN_reports_those_outside_a_fake_package(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .test("com.example.jail.data.fake", "FakeJailRepository.kt", "class FakeJailRepository")
            .test("com.example.jail.data", "FakeBankRepository.kt", "class FakeBankRepository")
            .test("com.example.jail", "FakeClock.kt", "object FakeClock\n\nclass Faker")
            .load(UnitKind.LIBRARY)

        val subjects = FakePlacement().check(sources).map { violation -> violation.subject }

        assertEquals(listOf("FakeBankRepository", "FakeClock"), subjects.sorted())
    }
}
