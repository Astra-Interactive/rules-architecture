@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class UseCaseAbstractionTest {

    @Test
    fun GIVEN_use_case_interfaces_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.rating.usecase/RatePlayerUseCase.kt", "interface RatePlayerUseCase")
            .main("x.rating.usecase/LowerRatingUseCase.kt", "abstract class LowerRatingUseCase")
            .main("x.rating.usecase/AddRatingUseCase.kt", "class AddRatingUseCase\n\nabstract class Base")
            .main("x.rating.api/RatingRepository.kt", "interface RatingRepository")
            .load(UnitKind.CONTEXT, rootPackage = "x")

        assertEquals(
            listOf(
                "Make LowerRatingUseCase a final class with operator fun invoke: a use case gets no interface; " +
                    "a test runs it with fakes of the contracts it takes.",
                "Make RatePlayerUseCase a final class with operator fun invoke: a use case gets no interface; " +
                    "a test runs it with fakes of the contracts it takes.",
            ),
            UseCaseAbstraction().check(sources).map(Violation::message).sorted()
        )
    }

    @Test
    fun GIVEN_use_case_interface_in_usecase_of_an_api_unit_WHEN_checked_THEN_reports_only_the_abstract_class(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.payment.api.offer.usecase/LoadOfferUseCase.kt", "interface LoadOfferUseCase")
            .main("x.payment.api.offer.usecase/BuyOfferUseCase.kt", "abstract class BuyOfferUseCase")
            .main("x.payment.api.offer.model/RefreshUseCase.kt", "interface RefreshUseCase")
            .load(UnitKind.API, rootPackage = "x.payment.api")

        assertEquals(
            listOf("BuyOfferUseCase", "RefreshUseCase"),
            UseCaseAbstraction().check(sources).map(Violation::subject).sorted()
        )
    }
}
