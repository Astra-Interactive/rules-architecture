@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.injection

import dev.detekt.api.Config
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.createEnvironment
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ConstructorDefaultDependencyTest {
    private val rule = ConstructorDefaultDependency(Config.empty)
    private val environment = createEnvironment()

    @Test
    fun GIVEN_primary_constructor_creating_dependencies_WHEN_linted_THEN_reports_each_parameter() {
        val code = """
            import kotlin.time.Clock

            object DefaultRepository

            class LivePredictionService(
                private val repo: MarketDataRepository,
                private val repository: Repository = DefaultRepository,
                private val pipeline: FeatureLabelPipeline = FeatureLabelPipeline(),
                private val model: DirectionModel = models.TemperatureCalibratedDirectionModel(),
                private val clock: Clock = Clock.System,
                private val qualifiedClock: Clock = kotlin.time.Clock.System
            )
        """.trimIndent()

        assertEquals(5, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_secondary_constructor_creating_dependency_WHEN_linted_THEN_reports_it() {
        val code = """
            class Pager(val source: Source) {
                constructor(size: Int, source: Source = HttpSource()) : this(source)
            }
        """.trimIndent()

        assertEquals(1, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_defaults_that_are_values_WHEN_linted_THEN_reports_nothing() {
        val code = """
            enum class Mode { Default }

            class Pager(
                val size: Int = 20,
                val items: List<String> = emptyList(),
                val factory: () -> Unit = {},
                val mode: Mode = Mode.Default,
                val formatter: Formatter = formatters.create()
            )
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_data_class_creating_default_WHEN_linted_THEN_leaves_it_to_data_class_rule() {
        val code = """
            data class Settings(val theme: Theme = Theme())
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_serializable_class_creating_default_WHEN_linted_THEN_reports_nothing() {
        val code = """
            import kotlinx.serialization.Serializable

            @Serializable
            class AstraRatingConfig(val killPlayer: KillPlayer = KillPlayer())
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }
}
