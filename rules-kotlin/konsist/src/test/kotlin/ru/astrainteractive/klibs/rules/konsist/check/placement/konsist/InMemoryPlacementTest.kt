@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
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

    @Test
    fun GIVEN_in_memory_class_in_a_memory_package_of_the_test_source_set_WHEN_checked_THEN_accepts_it(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.data.dao.rating.api/RatingRepository.kt", "interface RatingRepository")
            .test(
                "x.data.dao.rating.memory/InMemoryRatingRepository.kt",
                "internal class InMemoryRatingRepository : RatingRepository"
            )
            .load(UnitKind.CONTEXT, unitPath = ":modules:data:dao")

        assertEquals(emptyList(), InMemoryPlacement().check(sources))
    }

    @Test
    fun GIVEN_in_memory_class_in_a_memory_package_of_test_fixtures_WHEN_checked_THEN_accepts_it(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.data.dao.rating.api/RatingRepository.kt", "interface RatingRepository")
            .test(
                "testFixtures",
                "x.data.dao.rating.memory",
                "InMemoryRatingRepository.kt",
                "class InMemoryRatingRepository : RatingRepository"
            )
            .loadSourceSets(
                UnitKind.CONTEXT,
                mainSourceSet = "main",
                testSourceSet = "testFixtures"
            )

        assertEquals(emptyList(), InMemoryPlacement().check(sources))
    }

    @Test
    fun GIVEN_in_memory_class_outside_a_memory_package_of_the_test_source_set_WHEN_checked_THEN_reports_it_there(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.data.dao.rating.api/RatingRepository.kt", "interface RatingRepository")
            .test(
                "x.data.dao.rating.fake/InMemoryRatingRepository.kt",
                "internal class InMemoryRatingRepository : RatingRepository"
            )
            .load(UnitKind.CONTEXT, unitPath = ":modules:data:dao")

        assertEquals(
            listOf(
                "Move InMemoryRatingRepository to the memory package of its concept in the test source set."
            ),
            InMemoryPlacement().check(sources).map(Violation::message)
        )
    }

    @Test
    fun GIVEN_in_memory_class_outside_a_memory_package_of_test_fixtures_WHEN_checked_THEN_reports_it_there(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.data.dao.rating.api/RatingRepository.kt", "interface RatingRepository")
            .test(
                "testFixtures",
                "x.data.dao.rating",
                "InMemoryRatingRepository.kt",
                "class InMemoryRatingRepository : RatingRepository"
            )
            .loadSourceSets(
                UnitKind.CONTEXT,
                mainSourceSet = "main",
                testSourceSet = "testFixtures"
            )

        assertEquals(
            listOf(
                "Move InMemoryRatingRepository to the memory package of its concept in the testFixtures source set."
            ),
            InMemoryPlacement().check(sources).map(Violation::message)
        )
    }

    @Test
    fun GIVEN_in_memory_class_outside_a_memory_package_of_an_unnamed_test_directory_WHEN_checked_THEN_reports_it_there(
        @TempDir directory: File
    ) {
        val tree = SourceTree(directory)
            .main("x.data.dao.rating.api/RatingRepository.kt", "interface RatingRepository")
            .test("x.data.dao.rating.fake/InMemoryRatingRepository.kt", "class InMemoryRatingRepository")
        val sources = UnitSources.load(
            kind = UnitKind.CONTEXT,
            rootPackage = "x.data.dao",
            unitPath = null,
            mainDirectories = listOf(tree.mainDirectory),
            testDirectories = listOf(tree.testDirectory),
            mainSourceSets = emptyMap(),
            testSourceSets = emptyMap(),
            projectDirectory = directory,
            vocabulary = SourceTree.NO_PROJECT_WORDS,
            dependencyDirectories = emptyList(),
            androidNamespace = null
        )

        assertEquals(
            listOf("Move InMemoryRatingRepository to the memory package of its concept in the same source set."),
            InMemoryPlacement().check(sources).map(Violation::message)
        )
    }

    @Test
    fun GIVEN_in_memory_class_outside_a_memory_package_of_main_WHEN_checked_THEN_reports_it(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.data.dao.rating.api/RatingRepository.kt", "interface RatingRepository")
            .main(
                "x.data.dao.rating.exposed/InMemoryRatingRepository.kt",
                "internal class InMemoryRatingRepository : RatingRepository"
            )
            .load(UnitKind.CONTEXT, unitPath = ":modules:data:dao")

        assertEquals(
            listOf("Move InMemoryRatingRepository to the memory package of its concept."),
            InMemoryPlacement().check(sources).map(Violation::message)
        )
    }

    @Test
    fun GIVEN_misplaced_in_memory_classes_in_main_and_test_WHEN_reported_THEN_no_message_calls_them_production_code(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.data.dao.rating.api/InMemoryRatingCache.kt", "class InMemoryRatingCache")
            .test("x.data.dao.rating.fake/InMemoryShop.kt", "class InMemoryShop")
            .load(UnitKind.CONTEXT, unitPath = ":modules:data:dao")

        val messages = InMemoryPlacement().check(sources).map(Violation::message)

        assertEquals(2, messages.size)
        assertEquals(emptyList(), messages.filter { message -> "production code" in message })
    }
}
