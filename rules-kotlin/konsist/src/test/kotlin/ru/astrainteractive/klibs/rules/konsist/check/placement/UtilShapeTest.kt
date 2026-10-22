@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class UtilShapeTest {

    @Test
    fun GIVEN_types_in_util_and_extensions_elsewhere_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.command.rating.util", "KPlayerExt.kt", "fun String.toPlayer(): String = this\n\nclass Formatter")
            .main("x.command.rating.util", "Holders.kt", "object Holder\n\ninterface Shape")
            .main(
                "x.command.rating.model",
                "RatingCommand.kt",
                """
                class RatingCommand

                fun RatingCommand.isValid(): Boolean = true

                fun Int.clamp(): Int = this
                """
            )
            .main("x.command.rating.command", "RatingExecutor.kt", "internal class RatingExecutor\n\nfun run() = Unit")
            .main("x.astrarating.other", "Outside.kt", "fun Int.outside(): Int = this")
            .load(UnitKind.CONTEXT, rootPackage = "x.command")

        val subjects = UtilShape().check(sources).map { violation -> violation.subject }.sorted()

        assertEquals(listOf("Formatter", "Holder", "Int.clamp", "RatingCommand.isValid", "Shape"), subjects)
    }

    @Test
    fun GIVEN_entry_point_extensions_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.gui.bukkit.slot", "RatingsSlot.kt", "internal fun SlotContext.ratingsSlot(): String = \"\"")
            .load(UnitKind.CONTEXT, rootPackage = "x.gui.bukkit")

        assertEquals(emptyList(), UtilShape().check(sources))
    }

    @Test
    fun GIVEN_rule_on_a_list_of_a_model_in_the_model_file_WHEN_checked_THEN_reports_it(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.gui.bukkit.placeholder.model",
                "Coloring.kt",
                """
                class Coloring(val rating: Int)

                fun Collection<Coloring>.findByRating(rating: Int): Coloring? = firstOrNull()
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.gui.bukkit")

        assertEquals(
            listOf("Collection.findByRating"),
            UtilShape().check(sources).map { violation -> violation.subject }
        )
    }

    @Test
    fun GIVEN_extensions_on_plain_collection_and_function_receivers_WHEN_reported_THEN_name_the_file_after_the_subject(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.gui.bukkit.placeholder.model",
                "Coloring.kt",
                """
                class Coloring(val rating: Int)

                fun Coloring.isPositive(): Boolean = rating > 0

                fun List<Coloring>.findByRating(rating: Int): Coloring? = firstOrNull()

                fun ((Coloring) -> Unit).runFor(coloring: Coloring) = invoke(coloring)
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.gui.bukkit")

        val destinations = UtilShape().check(sources)
            .map { violation -> violation.message.substringAfter(" to ").substringBefore(" of the level") }
            .sorted()

        assertEquals(listOf("util/ColoringExt.kt", "util/ColoringExt.kt", "util/ColoringExt.kt"), destinations)
    }
}
