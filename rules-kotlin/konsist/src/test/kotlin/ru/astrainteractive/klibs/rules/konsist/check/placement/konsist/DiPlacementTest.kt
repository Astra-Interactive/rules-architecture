@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.baseName
import java.io.File
import kotlin.test.assertEquals

internal class DiPlacementTest {

    @Test
    fun GIVEN_di_modules_WHEN_checked_THEN_reports_those_outside_a_di_package(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.gui.api.di",
                "GuiModule.kt",
                "class GuiModule"
            )
            .main(
                "x.gui.api.rating.players.di",
                "RatingPlayersModule.kt",
                "class RatingPlayersModule\n\nobject PlayersWiringModule"
            )
            .main(
                "x.gui.api.rating.model",
                "StorageModule.kt",
                "class StorageModule"
            )
            .main(
                "x.gui.api.rating.component",
                "RatingComponent.kt",
                "object ComponentModule"
            )
            .load(UnitKind.API, unitPath = ":gui:api")

        val subjects = DiPlacement().check(sources).map(Violation::subject).sorted()

        assertEquals(listOf("ComponentModule", "StorageModule"), subjects)
    }

    @Test
    fun GIVEN_fake_di_module_in_test_sources_WHEN_checked_THEN_reports_none(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.gui.api.di/GuiModule.kt", "interface GuiModule")
            .test(
                "x.gui.api.fake",
                "FakeGuiModule.kt",
                """
                import x.gui.api.di.GuiModule

                class FakeGuiModule : GuiModule
                """
            )
            .load(UnitKind.API, unitPath = ":gui:api")

        assertEquals(emptyList(), DiPlacement().check(sources))
    }

    @Test
    fun GIVEN_fake_di_module_in_test_fixtures_sources_WHEN_checked_THEN_reports_none(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "main",
                "x.gui.api.di",
                "GuiModule.kt",
                "interface GuiModule"
            )
            .test(
                "testFixtures",
                "x.gui.api.fake",
                "FakeSharedGuiModule.kt",
                """
                import x.gui.api.di.GuiModule

                class FakeSharedGuiModule : GuiModule
                """
            )
            .loadSourceSets(
                UnitKind.API,
                mainSourceSet = "main",
                testSourceSet = "testFixtures"
            )

        assertEquals(listOf("FakeSharedGuiModule"), sources.testFiles.map(KoFileDeclaration::baseName))
        assertEquals(emptyList(), DiPlacement().check(sources))
    }

    @Test
    fun GIVEN_fake_unit_WHEN_checked_THEN_accepts_a_module_in_its_fake_package_and_reports_one_outside_it(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "commonMain",
                "com.example.payment.fake.offer.fake",
                "FakeOfferModule.kt",
                "class FakeOfferModule"
            )
            .main(
                "commonMain",
                "com.example.payment.fake.plan.model",
                "PlanModule.kt",
                "class PlanModule"
            )
            .loadSourceSets(
                UnitKind.FAKE,
                mainSourceSet = "commonMain",
                testSourceSet = "commonTest"
            )

        val subjects = DiPlacement().check(sources).map(Violation::subject)

        assertEquals(listOf("PlanModule"), subjects)
    }
}
