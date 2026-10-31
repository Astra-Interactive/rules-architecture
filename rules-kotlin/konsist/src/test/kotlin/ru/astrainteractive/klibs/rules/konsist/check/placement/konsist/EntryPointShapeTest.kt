@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Vocabulary
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

    @Test
    fun GIVEN_platform_classes_outside_the_root_package_WHEN_checked_THEN_sends_them_to_the_root(
        @TempDir directory: File
    ) {
        val vocabulary = Vocabulary(
            entryPointWords = emptySet(),
            technologyWords = emptySet(),
            pluralConcepts = emptySet(),
            platformClasses = setOf("WatchFaceService")
        )
        val sources = SourceTree(directory)
            .main("x.mobile/MainActivity.kt", "class MainActivity : ComponentActivity()")
            .main("x.mobile.application/App.kt", "class App : Application()")
            .main("x.mobile.work/PingWorker.kt", "class PingWorker(c: Context, p: P) : CoroutineWorker(c, p)")
            .main("x.mobile.face/Face.kt", "class Face : WatchFaceService()")
            .main("x.mobile.service/StatusNotificationFactory.kt", "class StatusNotificationFactory")
            .load(
                UnitKind.ENTRY_POINT,
                rootPackage = "x.mobile",
                vocabulary = vocabulary
            )

        val messages = EntryPointShape().check(sources).map { violation -> violation.message.substringBefore(':') }

        assertEquals(
            listOf(
                "Move App to the root package of the entry point",
                "Move Face to the root package of the entry point",
                "Move PingWorker to the root package of the entry point",
                "Move StatusNotificationFactory out of the entry point"
            ),
            messages.sorted()
        )
    }

    @Test
    fun GIVEN_horologist_tile_service_outside_the_root_WHEN_checked_THEN_sends_it_to_the_root(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.wear.tile/StatusesTileService.kt", "class StatusesTileService : SuspendingTileService()")
            .load(UnitKind.ENTRY_POINT, rootPackage = "x.wear")

        assertEquals(
            listOf("Move StatusesTileService to the root package of the entry point"),
            EntryPointShape().check(sources).map { violation -> violation.message.substringBefore(':') }
        )
    }
}
