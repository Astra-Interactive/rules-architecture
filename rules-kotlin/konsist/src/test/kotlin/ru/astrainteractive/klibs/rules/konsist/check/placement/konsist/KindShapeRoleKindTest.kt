@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File
import kotlin.test.assertEquals

internal class KindShapeRoleKindTest {

    @Test
    fun GIVEN_configured_role_kind_packages_under_a_concept_and_at_the_root_WHEN_checked_THEN_accepts_their_classes(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.vote.vote.limiter/VoteCooldownLimiter.kt", "internal class VoteCooldownLimiter")
            .main("x.vote.limiter/RequestLimiter.kt", "internal class RequestLimiter")
            .load(
                UnitKind.CONTEXT,
                rootPackage = "x.vote",
                vocabulary = ROLE_KIND_WORDS
            )

        assertEquals(expected = emptyList(), actual = KindShape().check(sources))
    }

    @Test
    fun GIVEN_role_kind_class_not_named_by_its_word_WHEN_checked_THEN_reports_only_that_class(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.vote.vote.limiter/VoteCooldownLimiter.kt",
                """
                internal class VoteCooldownLimiter {
                    private data class Entry(val count: Int)
                }
                """
            )
            .main("x.vote.vote.limiter/CooldownGate.kt", "internal class CooldownGate")
            .load(
                UnitKind.CONTEXT,
                rootPackage = "x.vote",
                vocabulary = ROLE_KIND_WORDS
            )

        assertEquals(
            expected = listOf(
                "Rename CooldownGate so that its name ends with Limiter: a role kind holds the classes its word " +
                    "names."
            ),
            actual = KindShape().check(sources).map(Violation::message)
        )
    }

    @Test
    fun GIVEN_function_interface_and_object_in_a_role_kind_WHEN_checked_THEN_reports_each_but_not_a_companion(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.vote.vote.limiter/VoteCooldownLimiter.kt",
                """
                internal class VoteCooldownLimiter {
                    companion object {
                        const val LIMIT = 3
                    }
                }
                """
            )
            .main("x.vote.vote.limiter/CooldownRules.kt", "internal fun isCoolingDown(): Boolean = false")
            .main("x.vote.vote.limiter/CooldownLimiter.kt", "internal interface CooldownLimiter")
            .main("x.vote.vote.limiter/DefaultLimiter.kt", "internal object DefaultLimiter")
            .load(
                UnitKind.CONTEXT,
                rootPackage = "x.vote",
                vocabulary = ROLE_KIND_WORDS
            )

        assertEquals(
            expected = listOf(
                "CooldownLimiter: Move CooldownLimiter out of 'vote/limiter': a role kind holds classes only.",
                "DefaultLimiter: Move DefaultLimiter out of 'vote/limiter': a role kind holds classes only.",
                "isCoolingDown: Move isCoolingDown out of 'vote/limiter': a role kind holds classes only.",
            ),
            actual = KindShape().check(sources).map { violation -> "${violation.subject}: ${violation.message}" }
                .sorted()
        )
    }

    @Test
    fun GIVEN_package_named_by_a_word_that_is_not_configured_WHEN_checked_THEN_reports_it_as_a_concept_package(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.gui.api.rating.codec/RatingCodec.kt", "internal class RatingCodec")
            .load(
                UnitKind.API,
                unitPath = ":gui:api",
                vocabulary = ROLE_KIND_WORDS
            )

        assertEquals(
            expected = listOf(
                "Move RatingCodec into a kind package of 'rating/codec': a concept package holds only packages."
            ),
            actual = KindShape().check(sources).map(Violation::message)
        )
    }

    @Test
    fun GIVEN_role_kind_package_in_an_app_api_unit_WHEN_checked_THEN_reports_it_as_a_kind_the_unit_may_not_hold(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.payment.api.offer.limiter/OfferRetryLimiter.kt", "class OfferRetryLimiter")
            .load(
                UnitKind.API,
                rootPackage = "x.payment.api",
                vocabulary = ROLE_KIND_WORDS,
                productKind = ProductKind.APP
            )

        assertEquals(
            expected = listOf(
                "Move OfferRetryLimiter out of 'offer/limiter': an app's api unit holds only the api, model, util, " +
                    "usecase, stub, di, component, viewmodel kinds, not limiter."
            ),
            actual = KindShape().check(sources).map(Violation::message)
        )
    }

    private companion object {
        val ROLE_KIND_WORDS = SourceTree.NO_PROJECT_WORDS.copy(roleKinds = setOf("limiter"))
    }
}
