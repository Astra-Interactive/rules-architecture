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

internal class TrivialLambdaReferenceFunctionTypeTest {
    private val rule = TrivialLambdaReference(Config.empty)
    private val environment = createEnvironment()

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
    fun GIVEN_plain_function_for_a_receiver_function_type_WHEN_linted_THEN_keeps_the_lambda(
        @TempDir directory: File
    ) {
        val original = """
            $RECEIVER_DECLARATIONS
            fun LazyListScope.faq(faqList: List<FaqDTO>) {
                items(faqList) { faqItem -> FaqItemComposable(faqItem) }
            }
        """.trimIndent()
        val reference = """
            $RECEIVER_DECLARATIONS
            fun LazyListScope.faq(faqList: List<FaqDTO>) {
                items(faqList, itemContent = ::FaqItemComposable)
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lintWithContext(environment, original).map { finding -> finding.message })
        val compilation = compile(reference, directory)
        assertTrue(compilation.exitCode != ExitCode.OK, compilation.messages)
    }

    @Test
    fun GIVEN_member_property_and_foreign_extension_for_a_receiver_function_type_WHEN_linted_THEN_keeps_the_lambdas() {
        val code = """
            $RECEIVER_DECLARATIONS
            fun Renderer.show(faqItem: FaqDTO) = Unit
            fun LazyListScope.faq(faqList: List<FaqDTO>, renderer: Renderer) {
                items(faqList) { faqItem -> renderer.render(faqItem) }
                items(faqList) { faqItem -> faqItem.open() }
                items(faqList) { faqItem -> faqItem.title }
                with(renderer) { items(faqList) { faqItem -> show(faqItem) } }
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lintWithContext(environment, code).map { finding -> finding.message })
    }

    @Test
    fun GIVEN_extension_of_the_expected_receiver_WHEN_linted_THEN_suggests_the_type_qualified_reference(
        @TempDir directory: File
    ) {
        val original = """
            $RECEIVER_DECLARATIONS
            fun LazyListScope.faq(faqList: List<FaqDTO>) {
                items(faqList) { faqItem -> FaqItem(faqItem) }
            }
        """.trimIndent()
        val reference = """
            $RECEIVER_DECLARATIONS
            fun LazyListScope.faq(faqList: List<FaqDTO>) {
                items(faqList, itemContent = LazyItemScope::FaqItem)
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "Replace the lambda with the function reference <Type>::FaqItem: the lambda only passes its " +
                    "parameters on."
            ),
            rule.lintWithContext(environment, original).map { finding -> finding.message }
        )
        val compilation = compile(reference, directory)
        assertEquals(
            ExitCode.OK,
            compilation.exitCode,
            compilation.messages
        )
    }

    @Test
    fun GIVEN_lambda_passed_to_an_unresolved_call_WHEN_linted_THEN_keeps_the_lambda() {
        val code = """
            class FaqDTO
            fun FaqItemComposable(faqItem: FaqDTO) = Unit
            fun faq(faqList: List<FaqDTO>) {
                items(faqList) { faqItem: FaqDTO -> FaqItemComposable(faqItem) }
                render(faqList, { faqItem: FaqDTO -> FaqItemComposable(faqItem) })
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lintWithContext(environment, code).map { finding -> finding.message })
    }

    @Test
    fun GIVEN_callback_that_drops_its_implicit_parameter_WHEN_linted_THEN_keeps_the_lambda(
        @TempDir directory: File
    ) {
        val declarations = "fun SettingsSwitchComposable(onCheckChange: (Boolean) -> Unit) = Unit"
        val original = """
            $declarations
            fun LessonSettingsSectionComposable(onToggleInfiniteRepeat: () -> Unit) {
                SettingsSwitchComposable(onCheckChange = { onToggleInfiniteRepeat.invoke() })
                val onCheck: (Boolean) -> Unit = { onToggleInfiniteRepeat.invoke() }
            }
        """.trimIndent()
        val reference = """
            $declarations
            fun LessonSettingsSectionComposable(onToggleInfiniteRepeat: () -> Unit) {
                SettingsSwitchComposable(onCheckChange = onToggleInfiniteRepeat::invoke)
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lintWithContext(environment, original).map { finding -> finding.message })
        val compilation = compile(reference, directory)
        assertTrue(compilation.exitCode != ExitCode.OK, compilation.messages)
    }

    private data class CompilationResult(val exitCode: ExitCode, val messages: String)

    private companion object {
        val RECEIVER_DECLARATIONS = """
            interface LazyItemScope
            interface LazyListScope
            class FaqDTO(val title: String) { fun open() = Unit }
            class Renderer { fun render(faqItem: FaqDTO) = Unit }
            fun <T> LazyListScope.items(
                items: List<T>,
                key: ((item: T) -> Any)? = null,
                itemContent: LazyItemScope.(item: T) -> Unit
            ) = Unit
            fun FaqItemComposable(faqItem: FaqDTO) = Unit
            fun LazyItemScope.FaqItem(faqItem: FaqDTO) = Unit
        """.trimIndent()
    }
}
