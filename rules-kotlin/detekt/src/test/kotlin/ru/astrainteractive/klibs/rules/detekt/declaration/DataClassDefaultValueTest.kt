@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.declaration

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class DataClassDefaultValueTest {
    private val rule = DataClassDefaultValue(Config.empty)

    @Test
    fun GIVEN_data_class_with_defaults_WHEN_linted_THEN_reports_each_defaulted_parameter() {
        val code = """
            data class PredictionInput(
                val symbol: String = "",
                val window: Int = 100,
                val features: List<Double>
            )
        """.trimIndent()

        assertEquals(2, rule.lint(code).size)
    }

    @Test
    fun GIVEN_serializable_data_class_with_defaults_WHEN_linted_THEN_reports_nothing() {
        val code = """
            import kotlinx.serialization.Serializable

            @Serializable
            data class PredictionInputDto(val symbol: String = "", val window: Int = 100)
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_regular_class_with_defaults_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class Pager(val pageSize: Int = 20)
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_data_class_without_defaults_WHEN_linted_THEN_reports_nothing() {
        val code = """
            data class PredictionInput(val symbol: String, val window: Int)
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
