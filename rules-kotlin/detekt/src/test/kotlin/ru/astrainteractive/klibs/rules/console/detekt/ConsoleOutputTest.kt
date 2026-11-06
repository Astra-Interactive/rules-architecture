@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.console.detekt

import dev.detekt.api.Config
import dev.detekt.test.lint
import dev.detekt.test.utils.compileForTest
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import java.io.File
import kotlin.test.assertEquals

internal class ConsoleOutputTest {
    private val rule = ConsoleOutput(Config.empty)

    private fun lintTexts(code: String): List<String> {
        return rule.lint(code).map { finding -> finding.entity.ktElement.text }
    }

    @Test
    fun GIVEN_println_and_print_stack_trace_on_failure_WHEN_linted_THEN_reports_both() {
        val code = """
            fun load(result: Result<Order>) {
                result.onFailure { error ->
                    println("Could not load order")
                    error.printStackTrace()
                }
            }
        """.trimIndent()

        assertEquals(
            listOf("""println("Could not load order")""", "error.printStackTrace()"),
            lintTexts(code)
        )
    }

    @Test
    fun GIVEN_failure_passed_to_the_logger_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun load(result: Result<Order>, logger: Logger) {
                result.onFailure { error -> logger.warn("Could not load order", error) }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_print_and_println_calls_WHEN_linted_THEN_reports_each() {
        val code = """
            fun report(order: Order) {
                print("Loading ")
                println(order.id)
                println()
                kotlin.io.println("Loaded")
            }
        """.trimIndent()

        val findings = rule.lint(code)

        assertEquals(
            listOf(
                """print("Loading ")""",
                "println(order.id)",
                "println()",
                """kotlin.io.println("Loaded")""",
            ),
            findings.map { finding -> finding.entity.ktElement.text }
        )
        assertEquals(
            "Write the message through the logger instead of println: the console is not a log. Only a " +
                "command-line entry point whose product is the console output suppresses ConsoleOutput, with " +
                "the reason next to it.",
            findings[1].message
        )
    }

    @Test
    fun GIVEN_uses_of_system_out_and_err_WHEN_linted_THEN_reports_each() {
        val code = """
            fun report(order: Order, bytes: ByteArray) {
                System.out.println(order)
                System.err.print("failed")
                System.out.printf("%s", order.id)
                java.lang.System.err.println(order)
                System.out.format("%s", order.id)
                System.err.write(bytes)
                System.out.append("done")
                System.err.flush()
                val writer = PrintWriter(System.out)
                val stream = PrintStream(System.err)
                val console = System.out
            }
        """.trimIndent()

        val findings = rule.lint(code)

        assertEquals(
            listOf(
                "System.out.println(order)",
                """System.err.print("failed")""",
                """System.out.printf("%s", order.id)""",
                "java.lang.System.err.println(order)",
                """System.out.format("%s", order.id)""",
                "System.err.write(bytes)",
                """System.out.append("done")""",
                "System.err.flush()",
                "System.out",
                "System.err",
                "System.out",
            ),
            findings.map { finding -> finding.entity.ktElement.text }
        )
        assertEquals(
            "Write the message through the logger instead of System.out: the console is not a log. Only a " +
                "command-line entry point whose product is the console output suppresses ConsoleOutput, with " +
                "the reason next to it.",
            findings.first().message
        )
    }

    @Test
    fun GIVEN_system_stream_wrapped_across_lines_WHEN_linted_THEN_reports_the_print() {
        val code = """
            fun report(order: Order) {
                System
                    .err
                    .println(order)
            }
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_print_stack_trace_without_arguments_WHEN_linted_THEN_reports_every_call_site() {
        val code = """
            fun load(loader: Loader, failure: Throwable?) {
                try {
                    loader.load()
                } catch (error: IOException) {
                    error.printStackTrace()
                }
                runCatching { loader.load() }.onFailure { throwable -> throwable.printStackTrace() }
                failure?.printStackTrace()
            }

            class LoadError(message: String) : Exception(message) {
                fun dump() {
                    printStackTrace()
                }
            }
        """.trimIndent()

        val findings = rule.lint(code)

        assertEquals(
            listOf(
                "error.printStackTrace()",
                "throwable.printStackTrace()",
                "failure?.printStackTrace()",
                "printStackTrace()",
            ),
            findings.map { finding -> finding.entity.ktElement.text }
        )
        assertEquals(
            "Pass the throwable to the logger instead of calling printStackTrace(): logger.warn(\"…\", error) " +
                "keeps the stack trace.",
            findings.first().message
        )
    }

    @Test
    fun GIVEN_thread_dump_stack_WHEN_linted_THEN_reports_each() {
        val code = """
            fun trace() {
                Thread.dumpStack()
                java.lang.Thread.dumpStack()
                val dump = Thread::dumpStack
            }
        """.trimIndent()

        val findings = rule.lint(code)

        assertEquals(
            listOf(
                "Thread.dumpStack()",
                "java.lang.Thread.dumpStack()",
                "Thread::dumpStack",
            ),
            findings.map { finding -> finding.entity.ktElement.text }
        )
        assertEquals(
            "Write the message through the logger instead of dumpStack: the console is not a log. Only a " +
                "command-line entry point whose product is the console output suppresses ConsoleOutput, with " +
                "the reason next to it.",
            findings.first().message
        )
    }

    @Test
    fun GIVEN_dump_stack_of_the_project_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class Diagnostics(private val logger: Logger, private val profiler: Profiler) {
                fun dumpStack() {
                    logger.warn("Stack", Throwable())
                }

                fun report() {
                    dumpStack()
                    profiler.dumpStack()
                }
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_logger_calls_and_writes_to_other_targets_WHEN_linted_THEN_reports_nothing() {
        val code = """
            fun load(loader: Loader, logger: Logger, writer: PrintWriter, lines: List<String>) {
                loader.load()
                    .onSuccess { order -> logger.info("Loaded order ${'$'}{order.id}") }
                    .onFailure { error -> logger.warn("Could not load order", error) }
                runCatching { loader.load() }.onFailure { error ->
                    Timber.e(error)
                    error.printStackTrace(writer)
                }
                writer.println("report")
                writer.print(lines.size)
                lines.forEach(writer::println)
                printf("%d", lines.size)
                terminal.out.println("ready")
            }

            fun Report.print(writer: PrintWriter) {
                writer.write(text)
            }
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_references_to_console_functions_WHEN_linted_THEN_reports_each() {
        val code = """
            fun report(lines: List<String>, failures: List<Throwable>) {
                lines.forEach(::println)
                lines.forEach(System.err::println)
                failures.forEach(Throwable::printStackTrace)
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "::println",
                "System.err::println",
                "Throwable::printStackTrace",
            ),
            lintTexts(code)
        )
    }

    @Test
    fun GIVEN_project_function_named_println_WHEN_called_unqualified_THEN_reports_the_call() {
        val code = """
            class Terminal(private val writer: PrintWriter) {
                fun println(line: String) {
                    writer.write(line)
                }

                fun greet() {
                    println("hello")
                }
            }
        """.trimIndent()

        assertEquals(listOf("""println("hello")"""), lintTexts(code))
    }

    @Test
    fun GIVEN_console_output_in_test_sources_WHEN_linted_THEN_reports_it(@TempDir directory: File) {
        val file = File(directory, "src/test/kotlin/OrderLoaderTest.kt")
        file.parentFile.mkdirs()
        file.writeText(
            """
            class OrderLoaderTest {
                fun GIVEN_order_WHEN_loaded_THEN_prints() {
                    println("loaded")
                }
            }
            """.trimIndent()
        )

        assertEquals(1, rule.lint(compileForTest(file.toPath())).size)
    }
}
