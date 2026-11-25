@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.direction.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File
import kotlin.test.assertEquals

internal class KindImportDirectionReferenceTest {

    private fun formatPaywallReports(sources: UnitSources): List<String> {
        return KindImportDirection().check(sources)
            .map { violation -> "${violation.path.substringAfter("x/paywall/impl/")} -> ${violation.subject}" }
            .sorted()
    }

    private fun SourceTree.withScreenImporters(): SourceTree {
        return main(
            "x.paywall.impl.offer.composable.screen",
            "OfferScreenComposable.kt",
            "import x.paywall.impl.composable.screen.splashScreen\n\nfun offerScreen() = Unit"
        )
            .main(
                "x.paywall.impl.component",
                "PaywallComponent.kt",
                "import x.paywall.impl.composable.screen.splashScreen\n\nclass PaywallComponent"
            )
            .main(
                "x.paywall.impl.router",
                "PaywallRouter.kt",
                "import x.paywall.impl.composable.screen.splashScreen\n\nclass PaywallRouter"
            )
            .main(
                "x.paywall.impl.di",
                "PaywallModule.kt",
                "import x.paywall.impl.composable.screen.splashScreen\n\nclass PaywallModule"
            )
            .main(
                "x.paywall.impl.util",
                "SplashExt.kt",
                "import x.paywall.impl.composable.screen.splashScreen\n\nfun showSplash() = Unit"
            )
    }

    @Test
    fun GIVEN_composable_sub_packages_WHEN_checked_THEN_they_import_each_other_and_only_hosts_import_a_screen(
        @TempDir
        directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.paywall.impl.composable.screen",
                "SplashScreenComposable.kt",
                """
                import x.paywall.impl.composable.component.heroCopy
                import x.paywall.impl.composable.layout.scrollFade
                import x.paywall.impl.composable.screen.splashContent

                fun splashScreen() = Unit
                """
            )
            .main(
                "x.paywall.impl.composable.screen",
                "SplashContentComposable.kt",
                "fun splashContent() = Unit"
            )
            .main(
                "x.paywall.impl.composable.component",
                "HeroCopyComposable.kt",
                """
                import x.paywall.impl.composable.layout.scrollFade
                import x.paywall.impl.composable.screen.splashScreen

                fun heroCopy() = Unit
                """
            )
            .main(
                "x.paywall.impl.composable.layout",
                "ScrollFadeComposable.kt",
                "fun scrollFade() = Unit"
            )
            .withScreenImporters()
            .load(UnitKind.CONTEXT, unitPath = ":paywall:impl")

        val reported = formatPaywallReports(sources)

        assertEquals(
            listOf(
                "composable/component/HeroCopyComposable.kt -> x.paywall.impl.composable.screen.splashScreen",
                "util/SplashExt.kt -> x.paywall.impl.composable.screen.splashScreen",
            ),
            reported
        )
    }

    @Test
    fun GIVEN_import_of_a_screen_WHEN_reported_THEN_message_names_what_may_host_it(
        @TempDir
        directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.paywall.impl.composable.screen",
                "SplashScreenComposable.kt",
                "fun splashScreen() = Unit"
            )
            .main(
                "x.paywall.impl.composable.component",
                "HeroCopyComposable.kt",
                "import x.paywall.impl.composable.screen.splashScreen\n\nfun heroCopy() = Unit"
            )
            .load(UnitKind.CONTEXT, unitPath = ":paywall:impl")

        assertEquals(
            "'composable/component' must not import x.paywall.impl.composable.screen.splashScreen from " +
                "'composable/screen': a screen is used only by what hosts or opens it, another screen, a " +
                "component's Render(), a router or di.",
            KindImportDirection().check(sources).single().message
        )
    }

    @Test
    fun GIVEN_fully_qualified_cross_kind_reference_WHEN_checked_THEN_reports_it(
        @TempDir
        directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.paywall.impl.model",
                "Paywall.kt",
                "class Paywall(val module: x.paywall.impl.di.PaywallModule)"
            )
            .main(
                "x.paywall.impl.di",
                "PaywallModule.kt",
                "class PaywallModule"
            )
            .load(UnitKind.CONTEXT, unitPath = ":paywall:impl")

        assertEquals(
            listOf("x.paywall.impl.di.PaywallModule"),
            KindImportDirection().check(sources).map(Violation::subject)
        )
    }

    @Test
    fun GIVEN_fake_references_WHEN_checked_THEN_accepts_contracts_models_and_rules_but_rejects_usecases(
        @TempDir
        directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.offer.api/OfferApi.kt", "interface OfferApi")
            .main("x.offer.model/Offer.kt", "class Offer")
            .main("x.offer.util/OfferExt.kt", "fun offerRule() = Unit")
            .main("x.offer.usecase/BuyOfferUseCase.kt", "class BuyOfferUseCase")
            .test(
                "x.offer.fake/FakeOfferApi.kt",
                """
                import x.offer.api.OfferApi
                import x.offer.model.Offer
                import x.offer.util.offerRule
                import x.offer.usecase.BuyOfferUseCase

                class FakeOfferApi : OfferApi
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.offer")

        assertEquals(
            listOf("x.offer.usecase.BuyOfferUseCase"),
            KindImportDirection().check(sources).map(Violation::subject)
        )
    }

    @Test
    fun GIVEN_framework_contract_fake_WHEN_checked_THEN_allows_required_signature_types_only(
        @TempDir
        directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.check.component/Check.kt",
                """
                interface Check {
                    val context: Context
                    fun check(input: Input): Output
                }
                class Context
                class Input
                class Output
                class Helper
                """
            )
            .test(
                "x.check.fake/FakeCheck.kt",
                """
                import x.check.component.Check
                import x.check.component.Context
                import x.check.component.Input as Value
                import x.check.component.Output
                import x.check.component.Helper

                class FakeCheck : Check {
                    override val context: Context = TODO()
                    override fun check(input: Value): Output = TODO()
                    val helper = Helper()
                }
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.check")

        assertEquals(
            listOf("x.check.component.Helper"),
            KindImportDirection().check(sources).map(Violation::subject)
        )
    }

    @Test
    fun GIVEN_a_feature_state_holder_WHEN_checked_THEN_it_depends_on_models_and_use_cases_but_not_di(
        @TempDir
        directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.app.home.feature/HomeFeature.kt",
                """
                import x.app.home.model.Home
                import x.app.home.usecase.LoadHomeUseCase

                class HomeFeature(val home: Home, val load: LoadHomeUseCase)
                """
            )
            .main(
                "x.app.home.feature/OtherFeature.kt",
                "import x.app.home.di.HomeModule\n\nclass OtherFeature(val module: HomeModule)"
            )
            .main("x.app.home.model/Home.kt", "class Home")
            .main("x.app.home.usecase/LoadHomeUseCase.kt", "class LoadHomeUseCase")
            .main("x.app.home.di/HomeModule.kt", "class HomeModule")
            .load(UnitKind.CONTEXT, rootPackage = "x.app")

        assertEquals(
            listOf(
                "'home/feature' must not import x.app.home.di.HomeModule from 'home/di': an entry point kind " +
                    "depends on model, contract, util, usecase, theme only."
            ),
            KindImportDirection().check(sources).map(Violation::message)
        )
    }
}
