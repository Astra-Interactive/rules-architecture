@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt

import dev.detekt.api.Config
import dev.detekt.core.config.YamlConfig
import dev.detekt.rules.style.DataClassContainsFunctions
import dev.detekt.rules.style.DataClassShouldBeImmutable
import dev.detekt.rules.style.ForbiddenComment
import dev.detekt.rules.style.ForbiddenImport
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
            import androidx.compose.runtime.Immutable
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

        assertEquals(10, findings.size)
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
}
