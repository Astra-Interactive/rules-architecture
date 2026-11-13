@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
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
            extensionKinds = setOf("slot"),
            roleKinds = emptySet()
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

    @Test
    fun GIVEN_private_composable_extensions_WHEN_receiver_belongs_to_project_THEN_requires_util(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.offer.model/Offer.kt", "data class Offer(val id: String)")
            .dependency("x.price.model/Price.kt", "data class Price(val amount: Int)")
            .main(
                "x.offer.composable/OfferComposable.kt",
                """
                import x.offer.model.Offer as Item
                import x.price.model.*
                import androidx.compose.ui.Modifier

                @Composable fun OfferComposable() = Unit
                private fun Item.title() = id
                private fun x.offer.model.Offer.subtitle() = id
                private fun Price.label() = amount.toString()
                private fun Modifier.card() = this
                private fun String.label() = this
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.offer")

        assertEquals(
            listOf(
                "Item.title",
                "Offer.subtitle",
                "Price.label"
            ),
            UtilShape().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_external_type_with_a_project_namesake_WHEN_imported_explicitly_THEN_keeps_private_extension(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.offer.model/Modifier.kt", "class Modifier")
            .main(
                "x.offer.composable/OfferComposable.kt",
                """
                import androidx.compose.ui.Modifier

                @Composable fun OfferComposable() = Unit
                private fun Modifier.card() = this
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.offer")

        assertEquals(emptyList(), UtilShape().check(sources))
    }

    @Test
    fun GIVEN_own_type_in_composable_package_WHEN_extended_privately_THEN_still_requires_util(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.offer.composable/Offer.kt", "class Offer")
            .main(
                "x.offer.composable/OfferComposable.kt",
                "@Composable fun OfferComposable() = Unit\nprivate fun Offer.title() = toString()"
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.offer")

        assertEquals(listOf("Offer.title"), UtilShape().check(sources).map(Violation::subject))
    }

    @Test
    fun GIVEN_util_functions_taking_a_project_contract_WHEN_checked_THEN_reports_each_form(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.vote.vote.api/VoteRepository.kt", "interface VoteRepository")
            .main("x.vote.vote.model/Vote.kt", "data class Vote(val id: Int)")
            .dependency("x.data.dao.rating.api/RatingRepository.kt", "interface RatingRepository")
            .main(
                "x.vote.vote.util/VoteRepositoryExt.kt",
                """
                import x.vote.vote.api.VoteRepository
                import x.vote.vote.model.Vote

                fun VoteRepository.loadVotes(): List<Vote> = emptyList()

                fun countVotes(repository: VoteRepository?): Int = 0

                fun sumVotes(repositories: List<VoteRepository>): Int = 0
                """
            )
            .main(
                "x.vote.vote.util/RatingExt.kt",
                """
                import x.data.dao.rating.api.*

                fun isRated(repository: RatingRepository): Boolean = true

                fun isVoted(repository: x.vote.vote.api.VoteRepository): Boolean = true
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.vote")

        val violations = UtilShape().check(sources)

        assertEquals(
            listOf(
                "VoteRepository.loadVotes",
                "countVotes",
                "isRated",
                "isVoted",
                "sumVotes"
            ),
            violations.map(Violation::subject).sorted()
        )
        assertEquals(
            "Move countVotes out of util: a util function takes no contract (VoteRepository); a flow over a " +
                "contract is a use case, a single call is made by the caller.",
            violations.first { violation -> violation.subject == "countVotes" }.message
        )
    }

    @Test
    fun GIVEN_util_functions_over_models_platform_and_library_types_WHEN_checked_THEN_reports_nothing(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.vote.vote.api/VoteRepository.kt", "interface VoteRepository")
            .main("x.vote.vote.model/Vote.kt", "data class Vote(val id: Int)")
            .main("x.vote.vote.model/Ballot.kt", "interface Ballot {\n    val id: Int\n}")
            .main(
                "x.vote.vote.util/VoteExt.kt",
                """
                import kotlinx.coroutines.flow.Flow
                import org.bukkit.entity.Player
                import x.vote.vote.model.Ballot
                import x.vote.vote.model.Vote

                fun Vote.isRecent(): Boolean = true

                fun Player.isVoter(votes: List<Vote>): Boolean = true

                fun Flow<Vote>.isEmpty(ballot: Ballot?): Boolean = true

                fun countVotes(repository: String): Int = 0
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.vote")

        assertEquals(emptyList(), UtilShape().check(sources))
    }
}
