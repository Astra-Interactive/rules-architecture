@file:Suppress("FunctionNaming", "FileLength")

package ru.astrainteractive.klibs.rules.konsist.check.direction.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class KindImportDirectionTest {

    private fun formatPaywallReports(sources: UnitSources): List<String> {
        return KindImportDirection().check(sources)
            .map { violation -> "${violation.path.substringAfter("x/paywall/impl/")} -> ${violation.subject}" }
            .sorted()
    }

    private fun SourceTree.withModelAndContracts(): SourceTree {
        return main(
            "x.gui.api.rating.model",
            "RatedPlayer.kt",
            """
            import x.gui.api.rating.api.*
            import x.gui.api.rating.util.isAllowed
            import kotlin.time.Duration

            data class RatedPlayer(val name: String, val cooldown: Duration, val other: Other)
            """
        )
            .main(
                "x.gui.api.rating.model",
                "Other.kt",
                "class Other"
            )
            .main(
                "x.gui.api.rating.api",
                "RatingPlayersRepository.kt",
                """
                import x.gui.api.rating.model.RatedPlayer
                import x.gui.api.rating.usecase.AddRatingUseCase

                interface RatingPlayersRepository
                """
            )
            .main(
                "x.gui.api.rating.util",
                "ConfigExt.kt",
                """
                import x.gui.api.rating.component.RatingComponent
                import x.gui.api.rating.model.RatedPlayer

                fun isAllowed(): Boolean = true
                """
            )
    }

    private fun SourceTree.withUseCasesAndImplementations(): SourceTree {
        return main(
            "x.gui.api.rating.usecase",
            "AddRatingUseCase.kt",
            """
            import x.gui.api.player.usecase.LoadPlayerUseCase
            import x.gui.api.rating.api.RatingPlayersRepository
            import x.gui.api.rating.component.RatingComponent
            import x.gui.api.rating.model.RatedPlayer
            import x.gui.api.rating.util.isAllowed

            class AddRatingUseCase
            """
        )
            .main(
                "x.gui.api.player.usecase",
                "LoadPlayerUseCase.kt",
                "class LoadPlayerUseCase"
            )
            .main(
                "x.gui.api.rating.memory",
                "InMemoryRatingPlayersRepository.kt",
                """
                import x.gui.api.rating.api.RatingPlayersRepository
                import x.gui.api.rating.component.RatingComponent
                import x.gui.api.rating.model.RatedPlayer

                class InMemoryRatingPlayersRepository
                """
            )
    }

    private fun SourceTree.withEntryPointsAndWiring(): SourceTree {
        return main(
            "x.gui.api.rating.component",
            "RatingComponent.kt",
            """
            import x.gui.api.di.GuiModule
            import x.gui.api.rating.Loose
            import x.gui.api.rating.api.RatingPlayersRepository
            import x.gui.api.rating.memory.InMemoryRatingPlayersRepository
            import x.gui.api.rating.slot.RatingSlot
            import x.gui.api.rating.usecase.AddRatingUseCase
            import x.gui.api.rating.util.isAllowed

            class RatingComponent
            """
        )
            .main(
                "x.gui.api.rating.slot",
                "RatingSlot.kt",
                "class RatingSlot"
            )
            .main(
                "x.gui.api.di",
                "GuiModule.kt",
                """
                import x.gui.api.rating.component.RatingComponent
                import x.gui.api.rating.memory.InMemoryRatingPlayersRepository

                class GuiModule
                """
            )
            .main(
                "x.gui.api.rating",
                "Loose.kt",
                """
                import x.gui.api.di.GuiModule

                class Loose
                """
            )
    }

    @Test
    fun GIVEN_unit_with_imports_against_the_direction_table_WHEN_checked_THEN_reports_each_one(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .withModelAndContracts()
            .withUseCasesAndImplementations()
            .withEntryPointsAndWiring()
            .load(UnitKind.API, unitPath = ":gui:api")

        val reported = KindImportDirection().check(sources)
            .map { violation -> "${violation.path.substringAfter("x/gui/api/")} -> ${violation.subject}" }
            .sorted()

        assertEquals(
            listOf(
                "rating/api/RatingPlayersRepository.kt -> x.gui.api.rating.usecase.AddRatingUseCase",
                "rating/component/RatingComponent.kt -> x.gui.api.di.GuiModule",
                "rating/component/RatingComponent.kt -> x.gui.api.rating.memory.InMemoryRatingPlayersRepository",
                "rating/memory/InMemoryRatingPlayersRepository.kt -> x.gui.api.rating.component.RatingComponent",
                "rating/model/RatedPlayer.kt -> x.gui.api.rating.api.*",
                "rating/model/RatedPlayer.kt -> x.gui.api.rating.util.isAllowed",
                "rating/usecase/AddRatingUseCase.kt -> x.gui.api.player.usecase.LoadPlayerUseCase",
                "rating/usecase/AddRatingUseCase.kt -> x.gui.api.rating.component.RatingComponent",
                "rating/util/ConfigExt.kt -> x.gui.api.rating.component.RatingComponent",
            ),
            reported
        )
    }

    @Test
    fun GIVEN_violation_WHEN_reported_THEN_message_names_both_places_and_the_allowed_kinds(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.jail.jail.model",
                "Jail.kt",
                "import x.jail.jail.api.JailRepository\n\nclass Jail"
            )
            .main(
                "x.jail.jail.api",
                "JailRepository.kt",
                "interface JailRepository"
            )
            .load(UnitKind.CONTEXT, unitPath = ":jail")

        val violation = KindImportDirection().check(sources).single()

        assertEquals("KindImportDirection", violation.rule)
        assertEquals(
            "'jail/model' must not import x.jail.jail.api.JailRepository from 'jail/api': " +
                "a model kind depends on model only.",
            violation.message
        )
    }

    @Test
    fun GIVEN_compose_theme_package_WHEN_checked_THEN_applies_the_theme_rows_of_the_direction_table(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.paywall.impl.theme",
                "PaywallMotion.kt",
                """
                import x.paywall.impl.composable.formatTitle
                import x.paywall.impl.model.Plan
                import x.paywall.impl.util.formatPrice

                internal object PaywallMotion
                """
            )
            .main(
                "x.paywall.impl.model",
                "Plan.kt",
                "class Plan"
            )
            .main(
                "x.paywall.impl.util",
                "PlanExt.kt",
                """
                import x.paywall.impl.theme.PaywallMotion

                fun formatPrice(): String = ""
                """
            )
            .main(
                "x.paywall.impl.composable",
                "HeroComposable.kt",
                """
                import x.paywall.impl.theme.PaywallMotion

                fun formatTitle(): String = ""
                """
            )
            .main(
                "x.paywall.impl.usecase",
                "LoadPlanUseCase.kt",
                "import x.paywall.impl.theme.PaywallMotion\n\nclass LoadPlanUseCase"
            )
            .load(UnitKind.CONTEXT, unitPath = ":paywall:impl")

        val reported = formatPaywallReports(sources)

        assertEquals(
            listOf(
                "theme/PaywallMotion.kt -> x.paywall.impl.composable.formatTitle",
                "theme/PaywallMotion.kt -> x.paywall.impl.util.formatPrice",
                "usecase/LoadPlanUseCase.kt -> x.paywall.impl.theme.PaywallMotion",
            ),
            reported
        )
    }

    @Test
    fun GIVEN_implementation_importing_util_and_its_table_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.jail.jail.file",
                "FileJailRepository.kt",
                """
                import x.jail.jail.api.JailRepository
                import x.jail.jail.file.table.JailTable
                import x.jail.jail.model.Jail
                import x.jail.jail.util.isSentenceOver

                internal class FileJailRepository
                """
            )
            .main(
                "x.jail.jail.file.table",
                "JailTable.kt",
                "object JailTable"
            )
            .main(
                "x.jail.jail.api",
                "JailRepository.kt",
                "interface JailRepository"
            )
            .main(
                "x.jail.jail.model",
                "Jail.kt",
                "class Jail"
            )
            .main(
                "x.jail.jail.util",
                "JailExt.kt",
                "fun isSentenceOver(): Boolean = false"
            )
            .load(UnitKind.CONTEXT, unitPath = ":jail")

        assertEquals(emptyList(), KindImportDirection().check(sources).map(Violation::subject))
    }

    private fun SourceTree.withUtilOverStorage(): SourceTree {
        return main(
            "x.data.exposed.rating.util",
            "RatingRowExt.kt",
            """
            import x.data.exposed.rating.database.RatingDatabase
            import x.data.exposed.rating.entity.RatingEntity
            import x.data.exposed.rating.table.RatingTable

            fun isRated(): Boolean = true
            """
        )
            .main(
                "x.data.exposed.rating.table",
                "RatingTable.kt",
                "object RatingTable"
            )
            .main(
                "x.data.exposed.rating.entity",
                "RatingEntity.kt",
                "class RatingEntity"
            )
            .main(
                "x.data.exposed.rating.database",
                "RatingDatabase.kt",
                "class RatingDatabase"
            )
    }

    @Test
    fun GIVEN_util_importing_table_entity_and_database_WHEN_unit_is_one_technology_THEN_reports_nothing(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .withUtilOverStorage()
            .load(UnitKind.TECHNOLOGY, unitPath = ":data:exposed")

        assertEquals(emptyList(), KindImportDirection().check(sources).map(Violation::subject))
    }

    @Test
    fun GIVEN_util_importing_table_and_entity_WHEN_unit_is_not_one_technology_THEN_reports_each(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .withUtilOverStorage()
            .load(UnitKind.CONTEXT, unitPath = ":data:exposed")

        assertEquals(
            listOf(
                "x.data.exposed.rating.entity.RatingEntity",
                "x.data.exposed.rating.table.RatingTable",
            ),
            KindImportDirection().check(sources).map(Violation::subject).sorted()
        )
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
        @TempDir directory: File
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
    fun GIVEN_import_of_a_screen_WHEN_reported_THEN_message_names_what_may_host_it(@TempDir directory: File) {
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
    fun GIVEN_fully_qualified_cross_kind_reference_WHEN_checked_THEN_reports_it(@TempDir directory: File) {
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
}
