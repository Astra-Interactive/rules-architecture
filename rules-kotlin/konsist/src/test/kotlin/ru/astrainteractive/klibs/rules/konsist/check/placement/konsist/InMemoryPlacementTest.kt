@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class InMemoryPlacementTest {

    @Test
    fun GIVEN_in_memory_implementations_WHEN_checked_THEN_reports_those_outside_a_memory_package(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.data.dao.rating.memory",
                "InMemoryRatingCache.kt",
                "class InMemoryRatingCache"
            )
            .main(
                "x.data.dao.rating.api",
                "InMemoryRatingRepository.kt",
                "class InMemoryRatingRepository"
            )
            .main(
                "x.data.dao.rating.model",
                "InMemoryness.kt",
                "class InMemoryness"
            )
            .test(
                "x.data.dao.rating.fake",
                "InMemoryShop.kt",
                "class InMemoryShop"
            )
            .load(UnitKind.CONTEXT, unitPath = ":modules:data:dao")

        val subjects = InMemoryPlacement().check(sources).map(Violation::subject)

        assertEquals(listOf("InMemoryRatingRepository", "InMemoryShop"), subjects)
    }

    @Test
    fun GIVEN_main_class_and_test_class_named_alike_WHEN_checked_THEN_reports_only_the_main_class(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.rating.command",
                "InMemoryPlacement.kt",
                "class InMemoryPlacement : ArchitectureCheck"
            )
            .test(
                "x.rating.command",
                "InMemoryPlacementTest.kt",
                "class InMemoryPlacementTest"
            )
            .load(UnitKind.CONTEXT, rootPackage = "x")

        assertEquals(listOf("InMemoryPlacement"), InMemoryPlacement().check(sources).map(Violation::subject))
    }
}
