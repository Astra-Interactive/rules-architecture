@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt

import dev.detekt.api.Config
import dev.detekt.core.config.YamlConfig
import dev.detekt.rules.coroutines.GlobalCoroutineUsage
import dev.detekt.rules.naming.PackageNaming
import dev.detekt.rules.style.DataClassContainsFunctions
import dev.detekt.rules.style.DataClassShouldBeImmutable
import dev.detekt.rules.style.ForbiddenComment
import dev.detekt.rules.style.ForbiddenImport
import dev.detekt.rules.style.MagicNumber
import dev.detekt.rules.style.WildcardImport
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class DetektArchitectureSettingsTest {
    private val settings: Config = loadSettings()
    private val styleConfig: Config = settings.subConfig("style")

    private fun loadSettings(): Config {
        val stream = checkNotNull(javaClass.classLoader.getResourceAsStream("detekt-architecture.yml")) {
            "detekt-architecture.yml is missing from the rule set jar"
        }
        return YamlConfig.load(stream.reader())
    }

    @Test
    fun GIVEN_region_and_separator_comments_WHEN_linted_THEN_reports_each() {
        val code = """
            // region Helpers
            fun first() = Unit
            //endregion

            // #region legacy
            // ------------
            // === Helpers ===
            /* ======= */
            // <editor-fold desc="Helpers">
            fun second() = Unit
        """.trimIndent()

        val findings = ForbiddenComment(styleConfig.subConfig("ForbiddenComment")).lint(code)

        assertEquals(7, findings.size)
    }

    @Test
    fun GIVEN_ordinary_comments_mentioning_regions_WHEN_linted_THEN_reports_nothing() {
        val code = """
            // Regional prices are cached for a day.
            // region-specific taxes are applied by the caller
            // Retries a - b -- c until the region answers.
            /**
             * Returns the region of the player.
             */
            fun region(): String = ""
        """.trimIndent()

        val findings = ForbiddenComment(styleConfig.subConfig("ForbiddenComment")).lint(code)

        assertEquals(0, findings.size)
    }

    @Test
    fun GIVEN_todo_marker_WHEN_linted_THEN_shared_marker_is_still_reported() {
        val code = """
            // TODO: remove after the migration
            fun first() = Unit
        """.trimIndent()

        val findings = ForbiddenComment(styleConfig.subConfig("ForbiddenComment")).lint(code)

        assertEquals(1, findings.size)
    }

    @Test
    fun GIVEN_kotlinx_datetime_instant_WHEN_linted_THEN_reports_it_and_allows_kotlin_time() {
        val code = """
            import kotlinx.datetime.Instant
            import kotlin.time.Duration
        """.trimIndent()

        val findings = ForbiddenImport(styleConfig.subConfig("ForbiddenImport")).lint(code)

        assertEquals(1, findings.size)
    }

    @Test
    fun GIVEN_platform_and_orm_imports_WHEN_linted_with_model_settings_THEN_reports_each() {
        val code = """
            import android.os.Parcelable
            import androidx.lifecycle.ViewModel
            import androidx.room.Entity
            import io.ktor.http.Url
            import io.papermc.paper.event.player.AsyncChatEvent
            import kotlinx.serialization.Serializable
            import net.fabricmc.api.ModInitializer
            import net.minecraft.world.entity.player.Player
            import net.minecraftforge.common.MinecraftForge
            import net.neoforged.bus.api.SubscribeEvent
            import org.bukkit.Location
            import org.jetbrains.exposed.v1.core.Table
        """.trimIndent()

        val findings = ForbiddenImport(styleConfig.subConfig("ForbiddenImport/model")).lint(code)

        assertEquals(11, findings.size)
    }

    @Test
    fun GIVEN_compose_imports_WHEN_linted_with_model_settings_THEN_reports_nothing() {
        val code = """
            import androidx.compose.runtime.Immutable
            import androidx.compose.ui.graphics.painter.Painter
            import org.jetbrains.compose.resources.StringResource
        """.trimIndent()

        val findings = ForbiddenImport(styleConfig.subConfig("ForbiddenImport/model")).lint(code)

        assertEquals(0, findings.size)
    }

    @Test
    fun GIVEN_model_settings_WHEN_read_THEN_excludes_only_technology_packages_not_any_data_path() {
        val excludes = styleConfig.subConfig("ForbiddenImport/model").valueOrNull<List<String>>("excludes")

        assertEquals(
            listOf(
                "**/exposed/**",
                "**/room/**",
                "**/ktor/**",
                "**/krate/**",
                "**/sqldelight/**",
                "**/datastore/**",
                "**/retrofit/**",
                "**/okhttp/**",
                "**/cache4k/**",
                "**/file/**",
                "**/memory/**",
                "**/table/**",
                "**/entity/**",
            ),
            excludes
        )
    }

    @Test
    fun GIVEN_compose_values_in_vals_and_inline_WHEN_linted_THEN_reports_only_the_bare_number() {
        val code = """
            private val PreviewCanvasColor = Color(0xFF1F1F1F)
            private val CardTopSpacing = 64.dp

            fun computeShift(offset: Int): Int = offset * 37
        """.trimIndent()

        val findings = MagicNumber(styleConfig.subConfig("MagicNumber")).lint(code)

        assertEquals(1, findings.size)
    }

    @Test
    fun GIVEN_callback_thread_and_di_framework_imports_WHEN_linted_THEN_reports_each() {
        val code = """
            import android.os.Handler
            import dagger.Module
            import dev.zacsweers.metro.Inject
            import java.util.Timer
            import java.util.TimerTask
            import javax.inject.Inject
            import kotlin.concurrent.thread
            import kotlin.coroutines.suspendCoroutine
            import kotlinx.coroutines.channels.trySendBlocking
            import kotlinx.coroutines.suspendCancellableCoroutine
            import org.koin.core.module.Module
        """.trimIndent()

        val findings = ForbiddenImport(styleConfig.subConfig("ForbiddenImport")).lint(code)

        assertEquals(9, findings.size)
    }

    @Test
    fun GIVEN_executor_and_future_imports_WHEN_linted_THEN_reports_each_outside_their_boundary() {
        val code = """
            import java.util.concurrent.CompletableFuture
            import java.util.concurrent.Executor
            import java.util.concurrent.Executors
        """.trimIndent()

        val executors = ForbiddenImport(styleConfig.subConfig("ForbiddenImport/executor")).lint(code)
        val futures = ForbiddenImport(styleConfig.subConfig("ForbiddenImport/future")).lint(code)

        assertEquals(2, executors.size)
        assertEquals(1, futures.size)
    }

    @Test
    fun GIVEN_resource_reads_WHEN_linted_with_resource_settings_THEN_reports_each_outside_util() {
        val code = """
            import androidx.compose.ui.res.painterResource
            import androidx.compose.ui.res.stringResource
            import org.jetbrains.compose.resources.painterResource
            import org.jetbrains.compose.resources.stringResource
            import org.jetbrains.compose.resources.vectorResource
        """.trimIndent()
        val resourceConfig = styleConfig.subConfig("ForbiddenImport/resource")

        val findings = ForbiddenImport(resourceConfig).lint(code)

        assertEquals(4, findings.size)
        assertEquals(listOf("**/util/**"), resourceConfig.valueOrNull<List<String>>("excludes"))
    }

    @Test
    fun GIVEN_coroutine_settings_WHEN_read_THEN_cancellation_rules_are_active() {
        val coroutines = settings.subConfig("coroutines")

        assertEquals(
            listOf(true, true),
            listOf("SuspendFunInFinallySection", "SuspendFunSwallowedCancellation")
                .map { rule -> coroutines.subConfig(rule).valueOrDefault("active", false) }
        )
    }

    @Test
    fun GIVEN_java_util_wildcard_WHEN_linted_THEN_reports_it() {
        val code = """
            import java.util.*
            import kotlin.time.Duration
        """.trimIndent()

        val findings = WildcardImport(styleConfig.subConfig("WildcardImport")).lint(code)

        assertEquals(1, findings.size)
    }

    @Test
    fun GIVEN_data_class_with_a_conversion_and_a_var_WHEN_linted_THEN_reports_both() {
        val code = """
            data class Plan(var price: Int) {
                fun toContent(): String = price.toString()
            }
        """.trimIndent()

        val functions = DataClassContainsFunctions(styleConfig.subConfig("DataClassContainsFunctions")).lint(code)
        val mutable = DataClassShouldBeImmutable(styleConfig.subConfig("DataClassShouldBeImmutable")).lint(code)

        assertEquals(2, functions.size + mutable.size)
    }

    @Test
    fun GIVEN_global_scope_WHEN_linted_THEN_reports_it() {
        val code = """
            import kotlinx.coroutines.GlobalScope
            import kotlinx.coroutines.launch

            fun start() {
                GlobalScope.launch { }
            }
        """.trimIndent()

        val findings = GlobalCoroutineUsage(settings.subConfig("coroutines").subConfig("GlobalCoroutineUsage"))
            .lint(code)

        assertEquals(1, findings.size)
    }

    @Test
    fun GIVEN_package_with_an_upper_case_letter_WHEN_linted_THEN_reports_it_and_allows_lowercase() {
        val rule = PackageNaming(settings.subConfig("naming").subConfig("PackageNaming"))

        assertEquals(1, rule.lint("package com.example.featureFlag").size)
        assertEquals(0, rule.lint("package com.example.featureflag2").size)
    }

    @Test
    fun GIVEN_old_time_imports_WHEN_linted_THEN_reports_each() {
        val code = """
            import java.util.Date
            import kotlinx.datetime.Clock
            import kotlin.time.Clock
        """.trimIndent()

        val findings = ForbiddenImport(styleConfig.subConfig("ForbiddenImport")).lint(code)

        assertEquals(2, findings.size)
    }

    @Test
    fun GIVEN_java_time_imports_WHEN_linted_with_time_settings_THEN_reports_each() {
        val code = """
            import java.time.Duration
            import java.time.Instant
            import java.time.LocalDateTime
            import java.util.Calendar
            import kotlin.time.Duration
        """.trimIndent()

        val findings = ForbiddenImport(styleConfig.subConfig("ForbiddenImport/time")).lint(code)

        assertEquals(4, findings.size)
    }
}
