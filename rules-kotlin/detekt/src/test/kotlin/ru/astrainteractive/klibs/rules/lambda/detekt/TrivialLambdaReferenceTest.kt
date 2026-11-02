@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.lambda.detekt

import dev.detekt.api.Config
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.createEnvironment
import org.jetbrains.kotlin.cli.common.ExitCode
import org.jetbrains.kotlin.cli.jvm.K2JVMCompiler
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.io.ByteArrayOutputStream
import java.io.File
import java.io.PrintStream
import kotlin.test.assertEquals
import kotlin.test.assertTrue

internal class TrivialLambdaReferenceTest {
    private val rule = TrivialLambdaReference(Config.empty)
    private val environment = createEnvironment()

    private data class CompilationResult(val exitCode: ExitCode, val messages: String)

    private fun compile(source: String, directory: File): CompilationResult {
        val sourceFile = directory.resolve("Fixture.kt").apply { writeText(source) }
        val outputDirectory = directory.resolve("classes").apply { mkdirs() }
        val messages = ByteArrayOutputStream()
        val result = PrintStream(messages).use { output ->
            K2JVMCompiler().exec(
                output,
                "-no-stdlib",
                "-classpath",
                System.getProperty("java.class.path"),
                "-d",
                outputDirectory.absolutePath,
                sourceFile.absolutePath
            )
        }
        return CompilationResult(result, messages.toString())
    }

    @Test
    fun GIVEN_unresolved_call_in_custom_suspend_lambda_WHEN_linted_THEN_avoids_an_unverified_reference() {
        val code = """
            fun produce(block: suspend () -> Unit) = Unit

            fun start() {
                produce {
                    listOf(1).forEach { value -> unresolved(value) }
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_resolved_constructor_WHEN_linted_THEN_suggests_its_reference() {
        val code = """
            class UserId(val value: Int)

            fun ids(values: List<Int>) = values.map { value -> UserId(value) }
        """.trimIndent()

        assertEquals(
            listOf(
                "Replace the lambda with the function reference ::UserId: the lambda only passes its parameters on."
            ),
            rule.lintWithContext(environment, code).map { finding -> finding.message }
        )
    }

    @Test
    fun GIVEN_emit_inside_on_start_WHEN_linted_THEN_keeps_the_inline_lambda() {
        val code = """
            import kotlinx.coroutines.flow.Flow
            import kotlinx.coroutines.flow.onStart

            fun start(values: Flow<Int>, initial: Int?): Flow<Int> = values.onStart {
                initial?.let { value -> emit(value) }
            }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_suspend_call_inside_custom_builder_WHEN_linted_THEN_keeps_the_inline_lambda() {
        val code = """
            import kotlinx.coroutines.flow.FlowCollector

            fun produce(block: suspend FlowCollector<Int>.() -> Unit) = Unit

            fun start() {
                produce {
                    listOf(1).forEach { value -> emit(value) }
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_bound_suspend_call_in_suspend_value_WHEN_linted_THEN_keeps_the_inline_lambda() {
        val code = """
            import kotlinx.coroutines.flow.FlowCollector

            fun action(collector: FlowCollector<Int>): suspend () -> Unit = {
                1.let { value -> collector.emit(value) }
            }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_suspend_function_value_passed_to_suspend_on_each_WHEN_linted_THEN_suggests_bound_invoke() {
        val code = """
            import kotlinx.coroutines.flow.Flow
            import kotlinx.coroutines.flow.onEach

            class Database

            fun configureDatabases(databases: Flow<Database>, configure: suspend (Database) -> Unit): Flow<Database> {
                return databases.onEach { database -> configure.invoke(database) }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Replace the lambda with the function reference configure::invoke: the lambda only passes its " +
                    "parameters on."
            ),
            rule.lintWithContext(environment, code).map { finding -> finding.message }
        )
    }

    @Test
    fun GIVEN_bound_suspend_reference_in_on_each_WHEN_compiled_THEN_is_accepted(@TempDir directory: File) {
        val source = """
            import kotlinx.coroutines.flow.Flow
            import kotlinx.coroutines.flow.onEach

            class Database

            fun configureDatabases(databases: Flow<Database>, configure: suspend (Database) -> Unit): Flow<Database> {
                return databases.onEach(configure::invoke)
            }
        """.trimIndent()

        val result = compile(source, directory)
        assertEquals(
            ExitCode.OK,
            result.exitCode,
            result.messages
        )
    }

    @Test
    fun GIVEN_suspend_reference_in_plain_inline_let_WHEN_compiled_THEN_is_rejected(@TempDir directory: File) {
        val source = """
            suspend fun emit(value: Int): Unit = Unit

            suspend fun send(): Unit {
                1.let(::emit)
            }
        """.trimIndent()

        val result = compile(source, directory)
        assertTrue(result.exitCode != ExitCode.OK, result.messages)
    }

    @Test
    fun GIVEN_named_suspend_and_generic_callees_WHEN_linted_THEN_suggests_supported_references(
        @TempDir directory: File
    ) {
        val declarations = """
            suspend fun load(value: Int): String = value.toString()
            suspend fun <T> identity(value: T): T = value
            fun <T, R> List<T>.mapSuspending(transform: suspend (T) -> R): List<R> = emptyList()
        """.trimIndent()
        val original = """
            $declarations
            fun convert(values: List<Int>): List<String> = values.mapSuspending { value -> load(value) }
            fun copy(values: List<Int>): List<Int> = values.mapSuspending { value -> identity(value) }
        """.trimIndent()
        val replacement = """
            $declarations
            fun convert(values: List<Int>): List<String> = values.mapSuspending(::load)
            fun copy(values: List<Int>): List<Int> = values.mapSuspending(::identity)
        """.trimIndent()

        assertEquals(
            listOf("::load", "::identity"),
            rule.lintWithContext(environment, original)
                .map { finding ->
                    finding.message.substringAfter("function reference ").substringBefore(": the lambda")
                }
        )
        val compilation = compile(replacement, directory)
        assertEquals(
            ExitCode.OK,
            compilation.exitCode,
            compilation.messages
        )
    }

    @Test
    fun GIVEN_unresolved_and_default_argument_calls_WHEN_linted_THEN_does_not_claim_a_reference() {
        val code = """
            fun send(value: Int, retry: Int = 0) = Unit
            fun run(values: List<Int>) {
                values.forEach { value -> send(value) }
                values.forEach { value -> unresolved(value) }
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lintWithContext(environment, code).map { finding -> finding.message })
    }

    @Test
    fun GIVEN_unresolved_property_WHEN_linted_THEN_does_not_claim_a_reference() {
        val code = """
            class User(val name: String)
            fun names(users: List<User>) = users.map { user -> user.missing }
        """.trimIndent()

        assertEquals(emptyList(), rule.lintWithContext(environment, code).map { finding -> finding.message })
    }

    @Test
    fun GIVEN_kstorage_overloaded_save_WHEN_linted_THEN_avoids_an_unproved_reference() {
        val code = """
            fun interface ValueSaver<T> { fun save(value: T) }
            interface MutableKrate<T> {
                fun save(value: T)
                fun save(block: (T) -> T)
            }
            fun <T : Any> MutableKrate<T?>.withDefault(): ValueSaver<T> {
                return ValueSaver { value -> save(value) }
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lintWithContext(environment, code).map { finding -> finding.message })
    }

    @Test
    fun GIVEN_vararg_callee_without_spread_WHEN_linted_THEN_conservatively_skips_a_compilable_reference(
        @TempDir originalDirectory: File,
        @TempDir referenceDirectory: File
    ) {
        val declarations = "fun append(vararg values: Int) = Unit"
        val original = """
            $declarations
            fun process(values: List<Int>) = values.forEach { value -> append(value) }
        """.trimIndent()
        val reference = """
            $declarations
            fun process(values: List<Int>) = values.forEach(::append)
        """.trimIndent()

        assertEquals(emptyList(), rule.lintWithContext(environment, original).map { finding -> finding.message })
        val originalCompilation = compile(original, originalDirectory)
        assertEquals(
            ExitCode.OK,
            originalCompilation.exitCode,
            originalCompilation.messages
        )
        val referenceCompilation = compile(reference, referenceDirectory)
        assertEquals(
            ExitCode.OK,
            referenceCompilation.exitCode,
            referenceCompilation.messages
        )
    }

    @Test
    fun GIVEN_bound_and_unbound_plain_references_WHEN_compiled_THEN_are_accepted(@TempDir directory: File) {
        val source = """
            class User(val name: String) {
                fun toContent(): String = name
            }
            class Repository {
                fun save(user: User) = Unit
            }
            fun convert(users: List<User>, repository: Repository) {
                users.map(User::toContent)
                users.map(User::name)
                users.forEach(repository::save)
            }
        """.trimIndent()

        val compilation = compile(source, directory)
        assertEquals(
            ExitCode.OK,
            compilation.exitCode,
            compilation.messages
        )
    }

    @Test
    fun GIVEN_plain_call_inside_on_start_WHEN_linted_THEN_still_suggests_a_reference() {
        val code = """
            import kotlinx.coroutines.flow.Flow
            import kotlinx.coroutines.flow.onStart

            fun format(value: Int): String = value.toString()

            fun start(values: Flow<Int>, initial: Int?): Flow<Int> = values.onStart {
                initial?.let { value -> format(value) }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Replace the lambda with the function reference ::format: the lambda only passes its parameters on."
            ),
            rule.lintWithContext(environment, code).map { finding -> finding.message }
        )
    }

    @Test
    fun GIVEN_lambdas_that_only_pass_their_parameters_WHEN_linted_THEN_reports_each_with_its_reference() {
        val code = """
            class UserId(val value: Long)
            class User(val name: String) {
                fun toContent(): String = name
            }
            interface Repository {
                fun save(user: User)
            }
            class PaywallViewModel {
                fun onPurchase() = Unit
                fun onClose() = Unit
            }
            fun format(value: Long): String = value.toString()
            fun parse(raw: String): Long = raw.toLong()
            fun Button(onClick: () -> Unit) = Unit

            fun render(ids: List<Long>, users: List<User>, repository: Repository, viewModel: PaywallViewModel) {
                ids.map { id -> UserId(id) }
                ids.map { id -> format(id) }
                users.map { user -> user.toContent() }
                users.map { user -> user.name }
                users.forEach { user -> repository.save(user) }
                ids.fold(0L) { total, id -> total.plus(id) }
                Button(onClick = { viewModel.onPurchase() })
                val onClose = { viewModel.onClose() }
            }

            val parseId = { raw: String -> parse(raw) }
        """.trimIndent()

        val messages = rule.lintWithContext(environment, code)
            .map { finding -> finding.message.substringBefore(": the lambda") }

        assertEquals(
            listOf(
                "Replace the lambda with the function reference ::UserId",
                "Replace the lambda with the function reference ::format",
                "Replace the lambda with the function reference <Type>::toContent",
                "Replace the lambda with the function reference <Type>::name",
                "Replace the lambda with the function reference repository::save",
                "Replace the lambda with the function reference <Type>::plus",
                "Replace the lambda with the function reference viewModel::onPurchase",
                "Replace the lambda with the function reference viewModel::onClose",
                "Replace the lambda with the function reference ::parse",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_lambdas_a_reference_cannot_replace_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun format(id: Long): String = ""
            fun format(id: Int): String = ""
            suspend fun loadOffer(id: Long): Int = 0

            fun render(ids: List<Long>, pairs: List<Pair<Int, Int>>, repository: Repository) {
                ids.map { id -> format(id) }
                ids.map { id -> loadOffer(id) }
                ids.map { id -> save(id, 1) }
                ids.map { id -> save(value = id) }
                ids.map { id -> save(*id) }
                ids.map { id -> save(id) { } }
                ids.map { id -> id.collect<Long>() }
                ids.map { id -> id.plus(id) }
                ids.map { id -> repository.save(id).also { } }
                ids.map { id -> id }
                ids.map { id -> listOf(id).first() }
                ids.map { id -> id.inc().dec() }
                pairs.map { (first, second) -> combine(first, second) }
                ids.forEach { repository.clear() }
                ids.map { id -> holder.repository.save(id) }
                ids.map { id -> id.value.save() }
                ids.map { id -> "id".save(id) }
                ids.map { id -> (save)(id) }
                ids.map { id -> }
                scope.launch { listOf(1).map { id -> repository.save(id) } }
            }

            suspend fun load(ids: List<Long>, repository: Repository) {
                ids.map { id -> repository.save(id) }
            }
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_coroutine_code_WHEN_linted_THEN_reports_only_lambdas_over_resolved_non_suspend_calls() {
        val code = """
            import kotlinx.coroutines.delay

            fun format(id: Long): String = ""

            suspend fun load(ids: List<Long>) {
                ids.map { id -> format(id) }
                ids.forEach { id -> delay(id) }
            }
        """.trimIndent()

        val messages = rule.lintWithContext(environment, code).map { finding -> finding.message }

        assertEquals(
            listOf(
                "Replace the lambda with the function reference ::format: the lambda only passes its parameters on.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_lambda_over_a_result_companion_function_WHEN_linted_THEN_suggests_the_imported_reference() {
        val code = """
            fun wrap(values: List<Int>) = values.map { value -> Result.success(value) }
        """.trimIndent()

        val messages = rule.lintWithContext(environment, code).map { finding -> finding.message }

        assertEquals(
            listOf(
                "Replace the lambda with the function reference ::success (import kotlin.Result.Companion.success): " +
                    "the lambda only passes its parameters on.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_member_and_top_level_extensions_WHEN_linted_THEN_reports_only_the_top_level_one() {
        val code = """
            class Shop(val name: String)

            fun Shop.exported(): Shop = this

            class FakeShopRepository(private val shops: List<Shop>) {
                private fun Shop.deepCopy(): Shop = Shop(name)

                fun copies(): List<Shop> = shops.map { shop -> shop.deepCopy() }

                fun exports(): List<Shop> = shops.map { shop -> shop.exported() }
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Replace the lambda with the function reference <Type>::exported: the lambda only passes its " +
                    "parameters on."
            ),
            rule.lintWithContext(environment, code).map { finding -> finding.message }
        )
    }

    @Test
    fun GIVEN_java_getter_read_as_a_property_WHEN_linted_THEN_reports_only_the_kotlin_property() {
        val code = """
            import java.util.logging.LogRecord

            class Shop(val name: String)

            fun levels(records: List<LogRecord>) = records.map { record -> record.level }

            fun names(shops: List<Shop>) = shops.map { shop -> shop.name }
        """.trimIndent()

        assertEquals(
            listOf(
                "Replace the lambda with the function reference <Type>::name: the lambda only passes its " +
                    "parameters on."
            ),
            rule.lintWithContext(environment, code).map { finding -> finding.message }
        )
    }

    @Test
    fun GIVEN_property_read_in_a_suspend_lambda_WHEN_linted_THEN_reports_only_the_plain_lambda() {
        val code = """
            class Message<T>(val value: T)

            fun <T, R> List<T>.mapSuspending(transform: suspend (T) -> R): List<R> = emptyList()

            fun suspending(messages: List<Message<String>>) = messages.mapSuspending { message -> message.value }

            fun plain(messages: List<Message<String>>) = messages.map { message -> message.value }
        """.trimIndent()

        assertEquals(
            listOf(
                "Replace the lambda with the function reference <Type>::value: the lambda only passes its " +
                    "parameters on."
            ),
            rule.lintWithContext(environment, code).map { finding -> finding.message }
        )
    }
}
