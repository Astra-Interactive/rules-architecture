@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.tuple.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class AnonymousTupleTest {
    private val rule = AnonymousTuple(Config.empty)

    @Test
    fun GIVEN_pair_and_triple_types_WHEN_linted_THEN_reports_each_type() {
        val code = """
            class Prices(val entries: List<Pair<String, Double>>)

            fun bounds(): Triple<Int, Int, Int> = TODO()

            fun first(prices: Prices): kotlin.Pair<String, Double> = prices.entries.first()
        """.trimIndent()

        assertEquals(3, rule.lint(code).size)
    }

    @Test
    fun GIVEN_pair_and_triple_constructed_by_name_WHEN_linted_THEN_reports_each_call() {
        val code = """
            fun entry() = Pair("price", 1.0)

            fun bounds() = Triple(1, 2, 3)
        """.trimIndent()

        assertEquals(2, rule.lint(code).size)
    }

    @Test
    fun GIVEN_to_outside_a_map_builder_WHEN_linted_THEN_reports_each_inferred_pair() {
        val code = """
            fun entry() = "price" to 1.0

            fun entries(prices: List<String>) = prices.associate { name -> name to 1.0 }

            fun pairs() = listOf("price" to 1.0)
        """.trimIndent()

        assertEquals(
            listOf("Replace Pair with a data class whose properties are named."),
            rule.lint(code).map(Finding::message).distinct()
        )
        assertEquals(3, rule.lint(code).size)
    }

    @Test
    fun GIVEN_to_inside_map_builders_and_map_entry_destructuring_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun prices(): Map<String, Double> = mapOf("price" to 1.0)

            fun cache(): MutableMap<String, Int> = mutableMapOf("hits" to 1, "misses" to 0)

            fun names(prices: Map<String, Double>) = prices.map { (name, value) -> name + value }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_named_data_class_WHEN_linted_THEN_reports_nothing() {
        val code = """
            data class Price(val name: String, val value: Double)

            fun price(): Price = Price("price", 1.0)

            fun invoke(factory: () -> Price) = factory()
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
