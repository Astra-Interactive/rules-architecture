@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File
import kotlin.test.assertEquals

internal class UtilShapeContractTest {

    @Test
    fun GIVEN_util_functions_over_data_types_nested_in_contracts_WHEN_checked_THEN_reports_only_a_nested_contract(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.vote.vote.api/VoteRepository.kt",
                """
                interface VoteRepository {
                    sealed interface Result {
                        data object Ok : Result
                    }
                    data object Empty
                    data class Snapshot(val count: Int)
                    enum class Mode { OPEN, CLOSED }
                    fun interface Factory {
                        fun create(): VoteRepository
                    }
                }
                """
            )
            .dependency(
                "x.data.dao.rating.api/RatingRepository.kt",
                """
                interface RatingRepository {
                    sealed interface Outcome {
                        interface Visitor
                    }
                    data object Missing
                }
                """
            )
            .main(
                "x.vote.vote.util/ResultExt.kt",
                """
                import x.vote.vote.api.VoteRepository.Result

                fun Result.isOk(): Boolean = this is Result.Ok

                fun isDone(result: Result?): Boolean = true
                """
            )
            .main(
                "x.vote.vote.util/VoteRepositoryExt.kt",
                """
                import x.data.dao.rating.api.RatingRepository
                import x.vote.vote.api.VoteRepository

                fun isEmpty(empty: VoteRepository.Empty, snapshot: VoteRepository.Snapshot): Boolean = true

                fun isOpen(mode: VoteRepository.Mode): Boolean = true

                fun isMissing(outcome: RatingRepository.Outcome, visitor: RatingRepository.Outcome.Visitor?) = true

                fun build(factory: VoteRepository.Factory): Int = 0
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.vote")

        assertEquals(listOf("build"), UtilShape().check(sources).map(Violation::subject))
    }

    @Test
    fun GIVEN_explicit_import_and_a_wildcard_import_of_a_contract_namesake_WHEN_checked_THEN_follows_the_explicit_one(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .dependency("x.data.dao.rating.api/Session.kt", "interface Session")
            .main(
                "x.vote.vote.util/SessionExt.kt",
                """
                import org.lib.Session
                import x.data.dao.rating.api.*

                fun isOpen(session: Session): Boolean = true
                """
            )
            .main(
                "x.vote.vote.util/TokenExt.kt",
                """
                import org.lib.Token as Session
                import x.data.dao.rating.api.*

                fun isClosed(session: Session?): Boolean = true
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.vote")

        assertEquals(emptyList(), UtilShape().check(sources))
    }

    @Test
    fun GIVEN_util_functions_with_a_context_parameter_of_a_contract_WHEN_checked_THEN_reports_them(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.vote.vote.api/VoteRepository.kt", "interface VoteRepository")
            .main(
                "x.vote.vote.util/VoteExt.kt",
                """
                import x.vote.vote.api.VoteRepository

                context(repository: VoteRepository)
                fun countVotes(): Int = 0

                context(locale: String, repositories: List<VoteRepository>)
                fun sumVotes(): Int = 0

                context(locale: String)
                fun formatVotes(): String = ""
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.vote")

        assertEquals(
            listOf("countVotes", "sumVotes"),
            UtilShape().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_util_functions_with_a_type_parameter_bounded_by_a_contract_WHEN_checked_THEN_reports_them(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.vote.vote.api/VoteRepository.kt", "interface VoteRepository")
            .main(
                "x.vote.vote.util/VoteExt.kt",
                """
                import x.vote.vote.api.VoteRepository

                fun <R : VoteRepository> countVotes(repository: R): Int = 0

                fun <R> sumVotes(repositories: List<R>): Int where R : VoteRepository = 0

                fun <R : VoteRepository> R.loadVotes(): Int = 0

                fun <T : Comparable<T>> maxVote(votes: List<T>): T = votes.first()
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.vote")

        assertEquals(
            listOf(
                "R.loadVotes",
                "countVotes",
                "sumVotes"
            ),
            UtilShape().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_util_functions_over_a_type_alias_of_a_contract_WHEN_checked_THEN_reports_them(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.vote.vote.api/VoteRepository.kt", "interface VoteRepository\n\ntypealias Votes = VoteRepository")
            .main("x.vote.vote.model/VoteCount.kt", "typealias VoteCount = Int")
            .dependency(
                "x.data.dao.rating.api/RatingRepository.kt",
                "interface RatingRepository\n\ntypealias Ratings = List<RatingRepository>"
            )
            .main(
                "x.vote.vote.util/VotesExt.kt",
                """
                import x.data.dao.rating.api.Ratings
                import x.vote.vote.api.Votes
                import x.vote.vote.model.VoteCount

                fun countVotes(votes: Votes): Int = 0

                fun Votes.sumVotes(): Int = 0

                fun isRated(ratings: Ratings): Boolean = true

                fun isEmpty(count: VoteCount): Boolean = true
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.vote")

        val violations = UtilShape().check(sources)

        assertEquals(
            listOf(
                "Votes.sumVotes",
                "countVotes",
                "isRated"
            ),
            violations.map(Violation::subject).sorted()
        )
        assertEquals(
            "Move countVotes out of util: a util function takes no contract (Votes); a flow over a contract is a " +
                "use case, a single call is made by the caller.",
            violations.first { violation -> violation.subject == "countVotes" }.message
        )
    }
}
