@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.tuple.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.KotlinAnalysisApiEngine
import dev.detekt.test.utils.KotlinEnvironmentContainer
import dev.detekt.test.utils.createEnvironment
import org.jetbrains.kotlin.cli.common.ExitCode
import org.jetbrains.kotlin.cli.jvm.K2JVMCompiler
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.PrintStream
import kotlin.test.assertEquals

internal class AnonymousTupleTest {
    private val rule = AnonymousTuple(Config.empty)
    private val environment = createEnvironment()

    private fun compileLibrary(directory: File): KotlinEnvironmentContainer {
        val sourceFile = directory.resolve("Library.kt").apply { writeText(LIBRARY) }
        val libraryJar = directory.resolve("library.jar")
        val messages = ByteArrayOutputStream()
        val exitCode = PrintStream(messages).use { output ->
            K2JVMCompiler().exec(
                output,
                "-no-stdlib",
                "-classpath",
                System.getProperty("java.class.path"),
                "-d",
                libraryJar.absolutePath,
                sourceFile.absolutePath
            )
        }
        assertEquals(
            ExitCode.OK,
            exitCode,
            messages.toString()
        )
        return KotlinEnvironmentContainer(
            javaSourceRoots = environment.javaSourceRoots,
            jvmClasspathRoots = environment.jvmClasspathRoots.plusElement(libraryJar.toPath())
        )
    }

    private fun lintReportedTexts(environment: KotlinEnvironmentContainer, code: String): List<String> {
        return rule.lintWithContext(environment, code)
            .sortedBy { finding -> finding.entity.location.source }
            .map { finding -> code.substring(finding.entity.location.text.start, finding.entity.location.text.end) }
    }

    private fun lintAgainstLibrary(directory: File, code: String): List<String> {
        val library = compileLibrary(directory)
        KotlinAnalysisApiEngine().use { engine ->
            engine.compile(
                code = code,
                jvmClasspathRoots = library.jvmClasspathRoots,
                allowCompilationErrors = false
            )
        }
        return lintReportedTexts(library, code)
    }

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

    @Test
    fun GIVEN_to_passed_to_library_pair_parameters_WHEN_linted_THEN_reports_nothing(@TempDir directory: File) {
        val code = """
            import acme.panels.Board
            import acme.panels.PanelsLayout
            import acme.panels.Serializer
            import acme.panels.panels
            fun named() = panels(source = "words", serializers = Serializer<Unit>() to Serializer<String>())
            fun positional() = panels("words", Serializer<Unit>() to Serializer<String>())
            fun namedLayout() = PanelsLayout(weights = 0.4F to 0.6F)
            fun positionalLayout() = PanelsLayout(0.4F to 0.6F)
            fun placed() = Board<Int>().place(cells = 1 to 2)
        """.trimIndent()

        assertEquals(emptyList(), lintAgainstLibrary(directory, code))
    }

    @Test
    fun GIVEN_pair_and_triple_constructed_for_library_tuple_parameters_WHEN_linted_THEN_reports_nothing(
        @TempDir directory: File
    ) {
        val code = """
            import acme.panels.PanelsLayout
            import acme.panels.Serializer
            import acme.panels.panels
            fun named() = panels(
                source = "words",
                serializers = Pair(Serializer<Unit>(), Serializer<String>()),
                spans = Triple(1, 2, 3)
            )
            fun positional() = panels("words", kotlin.Pair(Serializer<Unit>(), Serializer<String>()))
            fun namedLayout() = PanelsLayout(weights = Pair(0.4F, 0.6F))
            fun positionalLayout() = PanelsLayout(Pair(0.4F, 0.6F))
        """.trimIndent()

        assertEquals(emptyList(), lintAgainstLibrary(directory, code))
    }

    @Test
    fun GIVEN_tuples_passed_to_project_pair_parameters_WHEN_linted_THEN_reports_declarations_and_arguments() {
        val code = """
            class Layout(val weights: Pair<Float, Float>)
            fun show(weights: Pair<Float, Float>) = weights.first
            fun namedLayout() = Layout(weights = 0.4F to 0.6F)
            fun positionalShow() = show(Pair(0.4F, 0.6F))
        """.trimIndent()

        assertEquals(
            listOf(
                "Pair<Float, Float>",
                "Pair<Float, Float>",
                "0.4F to 0.6F",
                "Pair(0.4F, 0.6F)"
            ),
            lintReportedTexts(environment, code)
        )
    }

    @Test
    fun GIVEN_pair_stored_in_property_and_local_before_library_call_WHEN_linted_THEN_reports_each_value(
        @TempDir directory: File
    ) {
        val code = """
            import acme.panels.PanelsLayout
            val defaultWeights = Pair(1F, 1F)
            fun layout(): PanelsLayout {
                val weights = 0.4F to 0.6F
                return PanelsLayout(weights)
            }
        """.trimIndent()

        assertEquals(listOf("Pair(1F, 1F)", "0.4F to 0.6F"), lintAgainstLibrary(directory, code))
    }

    @Test
    fun GIVEN_tuples_passed_to_library_generic_and_list_parameters_WHEN_linted_THEN_reports_each_tuple(
        @TempDir directory: File
    ) {
        val code = """
            import acme.panels.remember
            import acme.panels.stack
            fun weights() = remember(0.4F to 0.6F)
            fun spans() = remember(Triple(1, 2, 3))
            fun stacked() = stack(listOf(0.4F to 0.6F))
        """.trimIndent()

        assertEquals(
            listOf(
                "0.4F to 0.6F",
                "Triple(1, 2, 3)",
                "0.4F to 0.6F"
            ),
            lintAgainstLibrary(directory, code)
        )
    }

    @Test
    fun GIVEN_tuples_passed_to_unresolved_call_WHEN_linted_THEN_reports_each_tuple() {
        val code = """
            fun weights() = missing(weights = 0.4F to 0.6F)
            fun spans() = missing(Pair(1, 2))
        """.trimIndent()

        val findings = rule.lintWithContext(
            environment,
            code,
            allowCompilationErrors = true
        )
        assertEquals(2, findings.size)
    }

    private companion object {
        val LIBRARY = """
            package acme.panels

            class Serializer<T>

            class PanelsLayout(val weights: Pair<Float, Float> = Pair(1F, 1F))

            class Board<T> {
                fun place(cells: Pair<T, T>): T = cells.first
            }

            fun <A, B> panels(
                source: String,
                serializers: Pair<Serializer<A>, Serializer<B>>?,
                spans: Triple<Int, Int, Int>? = null
            ): String = source

            fun <T> remember(value: T): T = value

            fun stack(weights: List<Pair<Float, Float>>): Int = weights.size
        """.trimIndent()
    }
}
