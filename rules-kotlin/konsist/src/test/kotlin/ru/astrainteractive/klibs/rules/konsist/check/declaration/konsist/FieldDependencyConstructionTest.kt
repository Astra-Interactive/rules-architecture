@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class FieldDependencyConstructionTest {

    @Test
    fun GIVEN_fields_constructing_dependencies_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.rating.api/RatingRepository.kt",
                "class ExposedRatingRepository(val database: Database)\n\nclass InMemoryRatingRepository"
            )
            .main("x.rating.model/Rating.kt", "class Rating")
            .main(
                "x.rating.command/RatingCommand.kt",
                """
                class RatingCommand {
                    private val repository = ExposedRatingRepository(Database())
                    private val lazyRepository by lazy { ExposedRatingRepository(Database()) }
                    private val rating = Rating()
                    private val name = "rating"
                    private val external = Json(true)
                    private val client = HttpClient(OkHttp)

                    fun reload() {
                        val repository = InMemoryRatingRepository()
                    }

                    fun start() {
                        InMemoryRatingRepository()
                    }
                }
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x")

        assertEquals(
            listOf(
                "RatingCommand.ExposedRatingRepository",
                "RatingCommand.Database",
                "RatingCommand.ExposedRatingRepository",
                "RatingCommand.Database",
                "RatingCommand.HttpClient",
                "RatingCommand.InMemoryRatingRepository",
                "RatingCommand.InMemoryRatingRepository"
            ),
            FieldDependencyConstruction().check(sources).map(Violation::subject)
        )
    }

    @Test
    fun GIVEN_platform_main_class_building_root_module_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x/AstraRating.kt",
                "class AstraRating : JavaPlugin() {\n    private val rootModule = RootModule(this)\n}"
            )
            .main("x.di/RootModule.kt", "internal class RootModule(plugin: JavaPlugin)")
            .load(UnitKind.ENTRY_POINT, rootPackage = "x")

        assertEquals(emptyList(), FieldDependencyConstruction().check(sources))
    }

    @Test
    fun GIVEN_di_module_constructing_dependencies_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.rating.api/RatingRepository.kt", "class ExposedRatingRepository(val database: Database)")
            .main(
                "x.rating.di/RatingModule.kt",
                "class RatingModule {\n    val repository = ExposedRatingRepository(Database())\n}"
            )
            .load(UnitKind.CONTEXT, rootPackage = "x")

        assertEquals(emptyList(), FieldDependencyConstruction().check(sources))
    }

    @Test
    fun GIVEN_class_extending_a_dependency_WHEN_checked_THEN_does_not_report_its_supertype(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.rating.task/ArchitectureTask.kt", "abstract class ArchitectureTask")
            .main("x.rating.task/ArchitectureCheckTask.kt", "class ArchitectureCheckTask : ArchitectureTask()")
            .load(UnitKind.CONTEXT, rootPackage = "x")

        assertEquals(emptyList(), FieldDependencyConstruction().check(sources))
    }
}
