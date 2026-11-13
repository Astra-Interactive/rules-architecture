@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.direction.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File
import kotlin.test.assertEquals

internal class KindImportDirectionUtilTest {

    @Test
    fun GIVEN_util_importing_a_contract_WHEN_checked_THEN_reports_it(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.jail.jail.api/JailRepository.kt", "interface JailRepository")
            .main(
                "x.jail.jail.util/JailExt.kt",
                "import x.jail.jail.api.JailRepository\n\nfun isSentenceOver(): Boolean = false"
            )
            .load(UnitKind.CONTEXT, unitPath = ":jail")

        assertEquals(
            listOf(
                "'jail/util' must not import x.jail.jail.api.JailRepository from 'jail/api': " +
                    "an util kind depends on model, util, theme only."
            ),
            KindImportDirection().check(sources).map(Violation::message)
        )
    }

    @Test
    fun GIVEN_util_importing_types_nested_in_a_contract_WHEN_checked_THEN_accepts_only_data_enum_and_sealed_types(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.jail.jail.api/JailRepository.kt",
                """
                interface JailRepository {
                    data class Snapshot(val count: Int)
                    enum class Mode { OPEN, CLOSED }
                    sealed interface Outcome {
                        data object Done : Outcome
                    }
                    sealed class Failure {
                        class Full : Failure()
                    }
                    class Session
                    fun interface Factory {
                        fun create(): JailRepository
                    }
                }
                """
            )
            .main(
                "x.jail.jail.util/JailExt.kt",
                """
                import x.jail.jail.api.JailRepository.Factory
                import x.jail.jail.api.JailRepository.Failure
                import x.jail.jail.api.JailRepository.Mode
                import x.jail.jail.api.JailRepository.Outcome
                import x.jail.jail.api.JailRepository.Outcome.Done
                import x.jail.jail.api.JailRepository.Session
                import x.jail.jail.api.JailRepository.Snapshot

                fun isSentenceOver(): Boolean = false
                """
            )
            .load(UnitKind.CONTEXT, unitPath = ":jail")

        assertEquals(
            listOf(
                "x.jail.jail.api.JailRepository.Factory",
                "x.jail.jail.api.JailRepository.Session",
            ),
            KindImportDirection().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_util_importing_objects_nested_in_a_contract_WHEN_checked_THEN_accepts_only_data_objects(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.vote.vote.api/VoteRepository.kt",
                """
                interface VoteRepository {
                    data object Empty
                    object Registry
                }
                """
            )
            .main(
                "x.vote.vote.util/EmptyExt.kt",
                """
                import x.vote.vote.api.VoteRepository.Empty
                import x.vote.vote.api.VoteRepository.Registry

                fun isEmpty(): Boolean = true
                """
            )
            .load(UnitKind.CONTEXT, unitPath = ":vote")

        assertEquals(
            listOf("x.vote.vote.api.VoteRepository.Registry"),
            KindImportDirection().check(sources).map(Violation::subject)
        )
    }

    @Test
    fun GIVEN_util_importing_model_util_and_theme_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.paywall.impl.model/Plan.kt", "class Plan")
            .main("x.paywall.impl.theme/PaywallMotion.kt", "internal object PaywallMotion")
            .main("x.paywall.impl.offer.util/OfferExt.kt", "fun isOffered(): Boolean = true")
            .main(
                "x.paywall.impl.util/PlanExt.kt",
                """
                import x.paywall.impl.model.Plan
                import x.paywall.impl.offer.util.isOffered
                import x.paywall.impl.theme.PaywallMotion

                fun formatPrice(): String = ""
                """
            )
            .load(UnitKind.CONTEXT, unitPath = ":paywall:impl")

        assertEquals(emptyList(), KindImportDirection().check(sources).map(Violation::subject))
    }
}
