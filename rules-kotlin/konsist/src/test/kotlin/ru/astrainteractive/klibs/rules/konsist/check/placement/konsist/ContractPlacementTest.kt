@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class ContractPlacementTest {

    @Test
    fun GIVEN_contracts_and_implementations_out_of_place_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.data.dao.rating.api",
                "RatingRepository.kt",
                "interface RatingRepository"
            )
            .main(
                "x.data.dao.rating.api",
                "CachedRatingRepository.kt",
                "class CachedRatingRepository : RatingRepository"
            )
            .main(
                "x.data.dao.rating.memory",
                "InMemoryRatingRepository.kt",
                "class InMemoryRatingRepository : RatingRepository"
            )
            .main(
                "x.data.dao.rating.model",
                "UserRepository.kt",
                "interface UserRepository"
            )
            .main(
                "x.data.dao.rating.remote",
                "HttpRating.kt",
                "class HttpRating"
            )
            .main(
                "x.data.dao.rating.remote",
                "HttpUser.kt",
                "class HttpUser"
            )
            .load(UnitKind.CONTEXT, unitPath = ":modules:data:dao")

        val subjects = ContractPlacement().check(sources).map(Violation::subject).sorted()

        assertEquals(
            listOf(
                "CachedRatingRepository",
                "UserRepository",
                "x.data.dao.rating.remote"
            ),
            subjects
        )
    }

    @Test
    fun GIVEN_interfaces_across_kinds_WHEN_checked_THEN_reports_each_outside_their_homes(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.offer.api/OfferApi.kt", "interface OfferGateway")
            .main("x.offer.usecase/LoadOfferUseCase.kt", "interface LoadOfferUseCase")
            .main("x.offer.component/OfferComponent.kt", "interface OfferComponent")
            .main("x.offer.viewmodel/OfferViewModel.kt", "interface OfferViewModel")
            .main("x.di/OfferModule.kt", "interface OfferModule")
            .main("x.offer.model/OfferResult.kt", "sealed interface OfferResult")
            .main("x.offer.model/Priced.kt", "interface Priced")
            .main("x.offer.ktor/Http.kt", "fun interface HttpEngine")
            .load(UnitKind.CONTEXT, rootPackage = "x")

        val messages = ContractPlacement().check(sources).map(Violation::message).sorted()

        assertEquals(
            listOf(
                "Move HttpEngine to the api package of its concept: a contract sits in api.",
                "Move Priced to the api package of its concept: a contract sits in api.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_state_holders_and_compose_scopes_WHEN_checked_THEN_sends_state_holders_to_their_kind(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.root.screen/RootScreenComponent.kt", "interface RootScreenComponent")
            .main("x.root.screen/RootRouter.kt", "interface RootRouter")
            .main("x.offer.api/OfferComponent.kt", "interface OfferComponent")
            .main("x.offer.composable/OfferViewModel.kt", "interface OfferViewModel")
            .main("x.root.component/RootComponent.kt", "interface RootComponent")
            .main(
                "x.filtercard.model/FilterCardScope.kt",
                "import androidx.compose.foundation.layout.ColumnScope\n\ninterface FilterCardScope : ColumnScope"
            )
            .main(
                "x.coroutine.model/WorkScope.kt",
                "import kotlinx.coroutines.CoroutineScope\n\ninterface WorkScope : CoroutineScope"
            )
            .load(UnitKind.CONTEXT, rootPackage = "x")

        val messages = ContractPlacement().check(sources).map(Violation::message).map { message ->
            message.substringBefore(':')
        }

        assertEquals(
            listOf(
                "Move OfferComponent to the component package of its concept",
                "Move OfferViewModel to the viewmodel package of its concept",
                "Move RootRouter to the api package of its concept",
                "Move RootScreenComponent to the component package of its concept",
                "Move WorkScope to the api package of its concept",
            ),
            messages.sorted()
        )
    }
}
