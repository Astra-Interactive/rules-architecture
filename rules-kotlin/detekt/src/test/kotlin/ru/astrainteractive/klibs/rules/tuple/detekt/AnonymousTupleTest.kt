@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.tuple.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.createEnvironment
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class AnonymousTupleTest {
    private val rule = AnonymousTuple(Config.empty)
    private val environment = createEnvironment()

    @Test
    fun GIVEN_pair_and_triple_types_WHEN_linted_THEN_reports_each_type() {
        val code = """
            class Prices(val entries: List<Pair<String, Double>>)

            fun bounds(): Triple<Int, Int, Int> = TODO()

            fun first(prices: Prices): kotlin.Pair<String, Double> = prices.entries.first()
        """.trimIndent()

        assertEquals(3, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_pair_and_triple_constructed_by_name_WHEN_linted_THEN_reports_each_call() {
        val code = """
            fun entry() = Pair("price", 1.0)

            fun bounds() = Triple(1, 2, 3)
        """.trimIndent()

        assertEquals(2, rule.lintWithContext(environment, code).size)
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
            rule.lintWithContext(environment, code).map(Finding::message).distinct()
        )
        assertEquals(3, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_to_inside_map_builders_and_map_entry_destructuring_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun prices(): Map<String, Double> = mapOf("price" to 1.0)

            fun cache(): MutableMap<String, Int> = mutableMapOf("hits" to 1, "misses" to 0)

            fun names(prices: Map<String, Double>) = prices.map { (name, value) -> name + value }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_named_data_class_WHEN_linted_THEN_reports_nothing() {
        val code = """
            data class Price(val name: String, val value: Double)

            fun price(): Price = Price("price", 1.0)

            fun invoke(factory: () -> Price) = factory()
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_custom_to_and_tuple_names_WHEN_resolved_THEN_does_not_report_domain_types() {
        val code = """
            package example
            class Pair(val value: String)
            class Triple(val value: String)
            class Status
            class Responses {
                infix fun Status.to(block: () -> Unit) { block() }
                fun configure(status: Status) { status to { println("OK") } }
            }
            infix fun String.to(other: Int): String = this + other
            fun label() = "size" to 4
            fun pair(): Pair = Pair("value")
            fun triple(): Triple = Triple("value")
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_aliased_tuple_types_and_constructors_WHEN_resolved_THEN_reports_them() {
        val code = """
            import kotlin.Pair as Duo
            import kotlin.Triple as Trio
            fun pair() = Duo("value", 1)
            fun triple() = Trio("value", 1, true)
            class Holder(val values: List<Duo<String, Int>>)
        """.trimIndent()

        assertEquals(3, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_custom_to_returning_a_real_pair_WHEN_resolved_THEN_reports_tuple() {
        val code = """
            class Source
            infix fun Source.to(value: Int) = kotlin.Pair(this, value)
            fun entry(source: Source) = source to 1
        """.trimIndent()

        assertEquals(2, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_unresolved_to_WHEN_resolved_THEN_does_not_guess_pair() {
        val code = "fun configure() = missing to unknown"

        val findings = rule.lintWithContext(
            environment,
            code,
            allowCompilationErrors = true
        )
        assertEquals(0, findings.size)
    }

    @Test
    fun GIVEN_shadowed_map_builder_WHEN_resolved_THEN_does_not_hide_pair() {
        val code = """
            fun mapOf(value: Any) = value
            fun values() = mapOf("x" to 1)
        """.trimIndent()

        assertEquals(1, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_typealias_and_nullable_tuple_WHEN_resolved_THEN_reports_actual_types() {
        val code = """
            typealias Coordinates = kotlin.Pair<Int, Int>
            fun coordinates(): Coordinates? = null
            fun origin() = Coordinates(0, 0)
        """.trimIndent()

        assertEquals(3, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_aliased_standard_map_builders_WHEN_resolved_THEN_allows_direct_entries_only() {
        val code = """
            import kotlin.collections.mapOf as dictionary
            fun first() = dictionary("x" to 1)
            fun second() = hashMapOf("x" to 1)
            fun third() = linkedMapOf("x" to 1)
            fun fourth() = sortedMapOf("x" to 1)
            fun nested() = dictionary("x" to (1 to 2))
        """.trimIndent()

        assertEquals(1, rule.lintWithContext(environment, code).size)
    }
}
