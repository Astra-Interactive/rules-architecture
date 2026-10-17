@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.injection

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class ConstructorDefaultDependencyTest {
    private val rule = ConstructorDefaultDependency(Config.empty)

    @Test
    fun GIVEN_primary_constructor_creating_dependencies_WHEN_linted_THEN_reports_each_parameter() {
        val code = """
            class LivePredictionService(
                private val repo: MarketDataRepository,
                private val pipeline: FeatureLabelPipeline = FeatureLabelPipeline(),
                private val model: DirectionModel = models.TemperatureCalibratedDirectionModel()
            )
        """.trimIndent()

        assertEquals(2, rule.lint(code).size)
    }

    @Test
    fun GIVEN_secondary_constructor_creating_dependency_WHEN_linted_THEN_reports_it() {
        val code = """
            class Pager(val source: Source) {
                constructor(size: Int, source: Source = HttpSource()) : this(source)
            }
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_defaults_that_are_values_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class Pager(
                val size: Int = 20,
                val items: List<String> = emptyList(),
                val factory: () -> Unit = {},
                val clock: Clock = Clock.System,
                val formatter: Formatter = formatters.create()
            )
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_data_class_creating_default_WHEN_linted_THEN_leaves_it_to_data_class_rule() {
        val code = """
            data class Settings(val theme: Theme = Theme())
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_serializable_class_creating_default_WHEN_linted_THEN_reports_nothing() {
        val code = """
            import kotlinx.serialization.Serializable

            @Serializable
            class AstraRatingConfig(val killPlayer: KillPlayer = KillPlayer())
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
