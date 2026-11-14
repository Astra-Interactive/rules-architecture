@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.direction.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File
import kotlin.test.assertEquals

internal class KindImportDirectionFakeTest {

    @Test
    fun GIVEN_fake_unit_WHEN_a_fake_imports_another_fake_of_the_unit_THEN_accepts_it(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "commonMain",
                "com.example.payment.fake.plan.fake",
                "FakePlanRepository.kt",
                "class FakePlanRepository"
            )
            .main(
                "commonMain",
                "com.example.payment.fake.offer.fake",
                "FakeOfferModule.kt",
                """
                import com.example.payment.fake.plan.fake.FakePlanRepository

                class FakeOfferModule {
                    val planRepository = FakePlanRepository()
                }
                """
            )
            .loadSourceSets(
                UnitKind.FAKE,
                mainSourceSet = "commonMain",
                testSourceSet = "commonTest"
            )

        assertEquals(emptyList(), KindImportDirection().check(sources))
    }

    @Test
    fun GIVEN_test_fakes_WHEN_one_imports_another_fake_of_the_test_tree_THEN_accepts_it(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.offer.api/OfferApi.kt", "interface OfferApi")
            .main("x.plan.api/PlanApi.kt", "interface PlanApi")
            .test(
                "x.plan.fake/FakePlanApi.kt",
                """
                import x.plan.api.PlanApi

                class FakePlanApi : PlanApi
                """
            )
            .test(
                "x.offer.fake/FakeOfferApi.kt",
                """
                import x.offer.api.OfferApi
                import x.plan.fake.FakePlanApi

                class FakeOfferApi : OfferApi {
                    val planApi = FakePlanApi()
                }
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x")

        assertEquals(emptyList(), KindImportDirection().check(sources))
    }

    @Test
    fun GIVEN_fake_of_a_di_module_interface_WHEN_checked_THEN_accepts_the_interface_and_rejects_another_di_class(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.offer.di/OfferModule.kt", "interface OfferModule")
            .main("x.offer.di/DefaultOfferModule.kt", "class DefaultOfferModule : OfferModule")
            .test(
                "x.offer.fake/FakeOfferModule.kt",
                """
                import x.offer.di.DefaultOfferModule
                import x.offer.di.OfferModule

                class FakeOfferModule : OfferModule {
                    val real = DefaultOfferModule()
                }
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.offer")

        assertEquals(
            listOf("x.offer.di.DefaultOfferModule"),
            KindImportDirection().check(sources).map(Violation::subject)
        )
    }
}
