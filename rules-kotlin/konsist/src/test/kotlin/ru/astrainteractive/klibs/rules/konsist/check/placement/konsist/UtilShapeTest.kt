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

internal class UtilShapeTest {

    @Test
    fun GIVEN_types_in_util_and_extensions_elsewhere_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.command.rating.util",
                "KPlayerExt.kt",
                "fun String.toPlayer(): String = this\n\nclass Formatter"
            )
            .main(
                "x.command.rating.util",
                "Holders.kt",
                "object Holder\n\ninterface Shape"
            )
            .main(
                "x.command.rating.model",
                "RatingCommand.kt",
                """
                class RatingCommand

                fun RatingCommand.isValid(): Boolean = true

                fun Int.clamp(): Int = this
                """
            )
            .main(
                "x.command.rating.command",
                "RatingExecutor.kt",
                "internal class RatingExecutor\n\nfun run() = Unit"
            )
            .main(
                "x.astrarating.other",
                "Outside.kt",
                "fun Int.outside(): Int = this"
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.command")

        val subjects = UtilShape().check(sources).map(Violation::subject).sorted()

        assertEquals(
            listOf(
                "Formatter",
                "Holder",
                "Int.clamp",
                "RatingCommand.isValid",
                "Shape"
            ),
            subjects
        )
    }

    @Test
    fun GIVEN_extensions_in_entry_point_kinds_WHEN_checked_THEN_reports_all_but_private_ones_of_a_composable_file(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.gui.bukkit.slot",
                "RatingsSlot.kt",
                "internal fun SlotContext.ratingsSlot(): String = \"\""
            )
            .main(
                "x.gui.bukkit.composable",
                "CardComposable.kt",
                """
                @Composable
                internal fun CardComposable() = Unit

                private fun Modifier.cardCanvas(): Modifier = this

                internal fun Modifier.sharedCanvas(): Modifier = this
                """
            )
            .main(
                "x.gui.bukkit.command",
                "RatingCommand.kt",
                "private fun KPlayer.toPlayerName(): String = name"
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.gui.bukkit")

        assertEquals(
            listOf(
                "KPlayer.toPlayerName",
                "Modifier.sharedCanvas",
                "SlotContext.ratingsSlot"
            ),
            UtilShape().check(sources).map(Violation::subject).sorted()
        )
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
            UtilShape().check(sources).map(Violation::subject)
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

        assertEquals(
            listOf(
                "util/ColoringExt.kt",
                "util/ColoringExt.kt",
                "util/ColoringExt.kt"
            ),
            destinations
        )
    }

    @Test
    fun GIVEN_slot_listed_as_an_extension_kind_WHEN_checked_THEN_accepts_its_extensions_only(@TempDir directory: File) {
        val vocabulary = Vocabulary(
            entryPointWords = emptySet(),
            technologyWords = emptySet(),
            pluralConcepts = emptySet(),
            platformClasses = emptySet(),
            extensionKinds = setOf("slot")
        )
        val sources = SourceTree(directory)
            .main(
                "x.gui.bukkit.slot",
                "EventSlot.kt",
                "internal fun SlotContext.killEventSlot(): String = \"\""
            )
            .main(
                "x.gui.bukkit.command",
                "RatingCommand.kt",
                "private fun KPlayer.toPlayerName(): String = name"
            )
            .load(
                UnitKind.CONTEXT,
                rootPackage = "x.gui.bukkit",
                vocabulary = vocabulary
            )

        assertEquals(
            listOf("KPlayer.toPlayerName"),
            UtilShape().check(sources).map(Violation::subject)
        )
    }
}
