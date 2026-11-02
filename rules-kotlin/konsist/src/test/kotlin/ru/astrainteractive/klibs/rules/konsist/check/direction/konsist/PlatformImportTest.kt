@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.direction.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File
import kotlin.test.assertEquals

internal class PlatformImportTest {
    private fun coreTree(directory: File): SourceTree {
        return SourceTree(directory)
            .main(
                "commonMain",
                "x.core.http.di",
                "HttpModule.kt",
                """
                import io.ktor.client.HttpClient
                import io.ktor.client.engine.HttpClientEngine
                import io.ktor.client.engine.HttpClientEngineFactory
                import io.ktor.serialization.kotlinx.json.json
                class HttpModule
                """
            )
            .main(
                "androidMain",
                "x.core.context.util",
                "ContextExt.kt",
                """
                import android.content.Context
                import androidx.core.content.ContextCompat
                fun Context.name() = packageName
                """
            )
            .main(
                "commonMain",
                "x.core.http.model",
                "HttpModel.kt",
                "import io.ktor.client.HttpClient\nclass HttpModel"
            )
            .main(
                "commonMain",
                "x.core.http.api",
                "HttpApi.kt",
                "import io.ktor.client.HttpClient\ninterface HttpApi"
            )
            .main(
                "commonMain",
                "x.core.http.usecase",
                "GetHttpUseCase.kt",
                "import io.ktor.client.HttpClient\nclass GetHttpUseCase"
            )
            .main(
                "commonMain",
                "x.core.context.util",
                "CommonContextExt.kt",
                "import android.content.Context\nimport androidx.core.content.ContextCompat\nclass CommonContextExt"
            )
            .main(
                "commonMain",
                "x.core.http.di",
                "ServerModule.kt",
                "import io.ktor.server.netty.Netty\nimport io.ktor.client.engine.android.Android\nclass ServerModule"
            )
    }

    private fun coreContext(directory: File): UnitContext {
        return UnitContext(
            UnitKind.CORE,
            ProductKind.APP,
            false,
            emptySet(),
            listOf(
                UnitContext.Compilation(
                    "androidJvm:android:main",
                    mapOf(
                        "commonMain" to setOf(File(directory, "src/commonMain/kotlin").absolutePath),
                        "androidMain" to setOf(File(directory, "src/androidMain/kotlin").absolutePath)
                    ),
                    false,
                    false,
                    emptySet()
                )
            )
        )
    }

    @Test
    fun GIVEN_core_common_di_and_android_util_WHEN_checked_THEN_allows_local_integrations(@TempDir directory: File) {
        val tree = coreTree(directory)
        val context = coreContext(directory)
        val sources = tree.loadMultipleSourceSets(
            UnitKind.CORE,
            context,
            "commonMain",
            "androidMain"
        )

        assertEquals(
            listOf(
                "android.content.Context",
                "androidx.core.content.ContextCompat",
                "io.ktor.client.HttpClient",
                "io.ktor.client.HttpClient",
                "io.ktor.client.HttpClient",
                "io.ktor.client.engine.android.Android",
                "io.ktor.server.netty.Netty"
            ),
            PlatformImport().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_custom_android_and_shared_source_sets_WHEN_checked_THEN_uses_compilation_membership(
        @TempDir directory: File
    ) {
        val tree = SourceTree(directory)
            .main(
                "androidSharedMain",
                "x.core.context.util",
                "AndroidContextExt.kt",
                "import android.content.Context\nfun Context.name() = packageName"
            )
            .main(
                "mobileMain",
                "x.core.context.util",
                "MobileContextExt.kt",
                "import android.content.Context\nclass MobileContextExt"
            )
        val android = File(directory, "src/androidSharedMain/kotlin").absolutePath
        val mobile = File(directory, "src/mobileMain/kotlin").absolutePath
        val context = UnitContext(
            UnitKind.CORE,
            ProductKind.APP,
            false,
            emptySet(),
            listOf(
                UnitContext.Compilation(
                    "androidJvm:android:main",
                    mapOf(
                        "androidSharedMain" to setOf(android),
                        "mobileMain" to setOf(mobile)
                    ),
                    false,
                    false,
                    emptySet()
                ),
                UnitContext.Compilation(
                    "native-ios_arm64:ios:main",
                    mapOf("mobileMain" to setOf(mobile)),
                    false,
                    false,
                    emptySet()
                )
            )
        )
        val sources = tree.loadMultipleSourceSets(
            UnitKind.CORE,
            context,
            "androidSharedMain",
            "mobileMain"
        )

        assertEquals(listOf("android.content.Context"), PlatformImport().check(sources).map(Violation::subject))
    }

    @Test
    fun GIVEN_platform_imports_in_model_api_and_usecase_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.jail.jail.model",
                "Jail.kt",
                "import org.bukkit.Location\n\nclass Jail"
            )
            .main(
                "x.jail.jail.api",
                "JailRepository.kt",
                "import io.ktor.http.*\n\ninterface JailRepository"
            )
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

        val subjects = PlatformImport().check(sources).map(Violation::subject).sorted()

        assertEquals(
            listOf(
                "io.ktor.http.*",
                "net.minecraft.world.Level",
                "org.bukkit.Location"
            ),
            subjects
        )
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
            PlatformImport().check(sources).map(Violation::subject)
        )
    }

    @Test
    fun GIVEN_api_unit_whose_component_declares_render_WHEN_checked_THEN_reports_only_the_other_platform_imports(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.payment.api.offer.component",
                "OfferComponent.kt",
                """
                import androidx.compose.runtime.Composable
                import androidx.lifecycle.ViewModel

                interface OfferComponent
                """
            )
            .main(
                "x.payment.api.offer.usecase",
                "LoadOfferUseCase.kt",
                "import androidx.compose.runtime.State\n\ninterface LoadOfferUseCase"
            )
            .load(
                UnitKind.API,
                unitPath = ":payment:api",
                compose = true
            )

        assertEquals(
            listOf("androidx.compose.runtime.State", "androidx.lifecycle.ViewModel"),
            PlatformImport().check(sources).map(Violation::subject).sorted()
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
            .load(
                UnitKind.IMPL,
                unitPath = ":paywall:impl",
                compose = true
            )

        val subjects = PlatformImport().check(sources).map(Violation::subject).sorted()

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
            PlatformImport().check(sources).map(Violation::subject)
        )
    }

    @Test
    fun GIVEN_fully_qualified_platform_types_WHEN_checked_THEN_reports_them_but_not_text(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.jail.jail.model",
                "Jail.kt",
                """
                // io.ktor.http.URLBuilder
                class Jail(
                    val location: org.bukkit.Location,
                    val server: org.spigotmc.Server
                ) {
                    val text = "io.papermc.paper.Server"
                }
                """
            )
            .load(UnitKind.CONTEXT, unitPath = ":jail")

        assertEquals(
            listOf("org.bukkit.Location", "org.spigotmc.Server"),
            PlatformImport().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_framework_exception_subclass_WHEN_in_mechanism_model_THEN_allows_only_the_base_type(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.command.rating.model/RatingCommandException.kt",
                """
                import org.bukkit.command.CommandException as FrameworkFailure
                import org.bukkit.Location
                import org.bukkit.command.OtherException

                class RatingCommandException : FrameworkFailure()
                """
            )
            .main(
                "x.command.rating.model/OtherCommandException.kt",
                "class OtherCommandException : org.bukkit.command.CommandException()"
            )
            .main(
                "x.command.rating.model/PlainCommandException.kt",
                "import org.bukkit.command.CommandException\nclass PlainCommandException : CommandException()"
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.command")

        assertEquals(
            listOf("org.bukkit.Location", "org.bukkit.command.OtherException"),
            PlatformImport().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_framework_exception_in_a_platform_free_api_WHEN_checked_THEN_keeps_the_boundary(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.command.api.rating.model/RatingCommandException.kt",
                "import org.bukkit.command.CommandException\nclass RatingCommandException : CommandException()"
            )
            .load(UnitKind.API, rootPackage = "x.command.api")

        assertEquals(
            listOf("org.bukkit.command.CommandException"),
            PlatformImport().check(sources).map(Violation::subject)
        )
    }

    @Test
    fun GIVEN_exception_type_in_contract_and_core_WHEN_checked_THEN_keeps_platform_types_out(
        @TempDir directory: File
    ) {
        val tree = SourceTree(directory)
            .main(
                "x.offer.api/OfferApi.kt",
                "import org.bukkit.command.CommandException\ninterface OfferApi { fun fail(): CommandException }"
            )
            .main("x.offer.theme/Theme.kt", "import androidx.lifecycle.ViewModel\nobject Theme")
            .main("x.outside/Other.kt", "import org.bukkit.Location\nclass Other")

        val context = tree.load(UnitKind.CONTEXT, rootPackage = "x.offer")
        assertEquals(
            listOf("org.bukkit.command.CommandException"),
            PlatformImport().check(context).map(Violation::subject)
        )
        val core = tree.load(UnitKind.CORE_API, rootPackage = "x.offer")
        assertEquals(3, PlatformImport().check(core).size)
    }
}
