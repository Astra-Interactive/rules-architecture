@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.direction

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class PlatformImportTest {

    @Test
    fun GIVEN_platform_imports_in_model_api_and_usecase_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.jail.jail.model", "Jail.kt", "import org.bukkit.Location\n\nclass Jail")
            .main("x.jail.jail.api", "JailRepository.kt", "import io.ktor.http.*\n\ninterface JailRepository")
            .main(
                "x.jail.jail.usecase",
                "JailPlayerUseCase.kt",
                "import net.minecraft.world.Level\n\nclass JailPlayerUseCase"
            )
            .main(
                "x.jail.jail.command",
                "JailCommand.kt",
                "import org.bukkit.command.Command\n\ninternal class JailCommand"
            )
            .main(
                "x.jail.jail.util",
                "LocationExt.kt",
                """
                import kotlin.time.Duration

                fun format(duration: Duration): String = duration.toString()
                """
            )
            .load(UnitKind.CONTEXT, unitPath = ":jail")

        val subjects = PlatformImport().check(sources).map { violation -> violation.subject }.sorted()

        assertEquals(listOf("io.ktor.http.*", "net.minecraft.world.Level", "org.bukkit.Location"), subjects)
    }

    @Test
    fun GIVEN_api_unit_WHEN_checked_THEN_every_file_is_platform_free(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.gui.api.rating.component",
                "RatingComponent.kt",
                "import org.bukkit.entity.Player\n\nclass RatingComponent"
            )
            .load(UnitKind.API, unitPath = ":gui:api")

        assertEquals(
            listOf("org.bukkit.entity.Player"),
            PlatformImport().check(sources).map { violation -> violation.subject }
        )
    }

    @Test
    fun GIVEN_compose_ui_unit_WHEN_checked_THEN_its_model_may_import_compose_but_no_other_platform(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.paywall.impl.model",
                "BenefitContent.kt",
                """
                import androidx.compose.ui.graphics.painter.Painter
                import androidx.room.Entity
                import org.jetbrains.compose.resources.StringResource

                class BenefitContent
                """
            )
            .main(
                "x.paywall.impl.api",
                "PaywallRepository.kt",
                "import androidx.compose.runtime.State\n\ninterface PaywallRepository"
            )
            .main(
                "x.paywall.impl.composable",
                "BenefitComposable.kt",
                "import androidx.compose.runtime.Composable\n\nfun benefit() = Unit"
            )
            .load(UnitKind.CONTEXT, unitPath = ":paywall:impl")

        val subjects = PlatformImport().check(sources).map { violation -> violation.subject }.sorted()

        assertEquals(listOf("androidx.compose.runtime.State", "androidx.room.Entity"), subjects)
    }

    @Test
    fun GIVEN_unit_without_composables_WHEN_model_imports_compose_THEN_reports_it(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.paywall.api.model",
                "Plan.kt",
                "import androidx.compose.ui.graphics.Color\n\nclass Plan"
            )
            .load(UnitKind.CONTEXT, unitPath = ":paywall:api")

        assertEquals(
            listOf("androidx.compose.ui.graphics.Color"),
            PlatformImport().check(sources).map { violation -> violation.subject }
        )
    }
}
