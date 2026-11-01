@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.injection.detekt

import dev.detekt.api.Config
import dev.detekt.core.config.YamlConfig
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.createEnvironment
import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.util.encode
import java.io.StringReader
import kotlin.test.assertEquals
import kotlin.test.assertTrue

internal class FieldDependencyConstructionTest {
    private val environment = createEnvironment()
    private val rule = FieldDependencyConstruction(Config.empty)

    @Test
    fun GIVEN_retained_and_called_dependencies_WHEN_checked_THEN_reports_constructors_only() {
        val code = """
            class Client { fun load(): String = "value" }
            fun Client(value: Int): String = value.toString()
            class Consumer {
                val client = Client()
                val deferred by lazy { Client() }
                fun load(): String {
                    val client = Client()
                    val alias = client
                    return alias.load()
                }
                fun inline(): String = Client().load()
                fun value(): String = Client(1)
            }
        """.trimIndent()

        val findings = rule.lintWithContext(environment, code)

        assertEquals(4, findings.size)
        assertTrue(findings.all { finding -> finding.message.contains("constructs Client") })
    }

    @Test
    fun GIVEN_returned_wrappers_and_aliases_WHEN_checked_THEN_reports_nothing() {
        val code = """
            package example

            class Player(val id: Int) { fun name(): String = id.toString() }
            class Server {
                fun direct(): Player = Player(1)
                fun qualified(): Player { return example.Player(2) }
                fun alias(): Player {
                    val player = Player(3)
                    val result = player
                    return (result)
                }
                fun method(id: Int): Player { return Player(id) }
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lintWithContext(environment, code))
    }

    @Test
    fun GIVEN_returned_instance_also_used_for_behavior_WHEN_checked_THEN_reports_dependency() {
        val code = """
            class Client { fun load() = Unit }
            class Consumer {
                fun create(): Client {
                    val result = Client()
                    result.load()
                    return result
                }
                fun mutable(): Client {
                    var result = Client()
                    result.load()
                    return result
                }
            }
        """.trimIndent()

        assertEquals(2, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_value_types_and_owned_builder_state_WHEN_checked_THEN_reports_nothing() {
        val code = """
            data class Text(val value: String)
            enum class Mode { Default }
            @JvmInline value class Id(val value: Int)
            class TextBuilder {
                private val translations = mutableMapOf<String, String>()
                fun translation(language: String, value: String) { translations[language] = value }
                fun build(): Text = Text(translations.values.joinToString())
            }
            class Counter {
                private var value = 0
                fun next(): Int { value += 1; return value }
            }
            class Consumer {
                private val mode = Mode.Default
                private val value = Text("a")
                private val id = Id(1)
                private val values = java.util.ArrayList<String>()
                private val counter = Counter()
                fun text(): Text = TextBuilder().apply { translation("en", "Hello") }.build()
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lintWithContext(environment, code))
    }

    @Test
    fun GIVEN_builder_using_external_behavior_WHEN_checked_THEN_does_not_exempt_it_by_name() {
        val code = """
            object Network { fun send() = Unit }
            class TextBuilder {
                private var text = ""
                fun build(): String { Network.send(); return text }
            }
            class Consumer {
                fun text(): String = TextBuilder().build()
            }
        """.trimIndent()

        assertEquals(1, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_di_wiring_and_constructor_defaults_WHEN_checked_THEN_leaves_them_to_their_own_rules() {
        val wiring = """
            package example.di
            class Client { fun load() = Unit }
            class Module { val client = Client() }
        """.trimIndent()
        val code = """
            class Client { fun load() = Unit }
            open class Parent
            class Consumer(val client: Client = Client()) : Parent()
            val topLevel = Client()
        """.trimIndent()

        assertEquals(emptyList(), rule.lintWithContext(environment, wiring))
        assertEquals(emptyList(), rule.lintWithContext(environment, code))
    }

    @Test
    fun GIVEN_entry_point_composition_WHEN_checked_THEN_allows_di_root_but_reports_other_dependencies() {
        val context = UnitContext(
            UnitKind.ENTRY_POINT,
            ProductKind.PLATFORM_PLUGIN,
            false,
            emptySet(),
            emptyList()
        )
        val config = YamlConfig.load(StringReader("unitContext: '${context.encode()}'"))
        val code = """
            package example
            class Client { fun load() = Unit }
            class Plugin {
                val root = example.di.RootModule()
                val client = Client()
            }
        """.trimIndent()
        val wiring = "package example.di\nclass RootModule { fun start() = Unit }"

        assertEquals(
            2,
            rule.lintWithContext(
                environment,
                code,
                wiring
            ).size
        )
        val findings = FieldDependencyConstruction(config).lintWithContext(
            environment,
            code,
            wiring
        )
        assertEquals(1, findings.size)
        assertTrue(findings.single().message.contains("constructs Client"))
    }

    @Test
    fun GIVEN_unresolved_calls_and_simple_holders_WHEN_checked_THEN_does_not_guess_a_dependency_from_its_name() {
        val code = """
            class Holder(val value: String)
            class Consumer {
                val unknown = UnknownClient()
                val holder = Holder("value")
                val text = "Client()"
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lintWithContext(environment, code))
    }
}
