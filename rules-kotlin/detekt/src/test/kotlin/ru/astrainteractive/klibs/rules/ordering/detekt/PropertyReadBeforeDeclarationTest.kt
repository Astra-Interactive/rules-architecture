@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.ordering.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.createEnvironment
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class PropertyReadBeforeDeclarationTest {
    private val rule = PropertyReadBeforeDeclaration(Config.empty)
    private val environment = createEnvironment()

    @Test
    fun GIVEN_reads_during_construction_WHEN_linted_THEN_reports_each() {
        val code = """
            class Scope(val name: String)

            class CoreModule {
                val title = "core " + name
                val delegated by Holder(scope)
                val viaThis = this.scope.name

                init {
                    println(scope)
                }

                private val name = "core"
                private val scope = Scope("main")
            }

            class Holder(private val value: Any) {
                operator fun getValue(thisRef: Any?, property: Any?): Any = value
            }
        """.trimIndent()

        assertEquals(
            listOf(
                "'name' is read before it is declared, while the object is still being constructed: declare " +
                    "'name' above this line or read it in code that runs later.",
                "'scope' is read before it is declared, while the object is still being constructed: declare " +
                    "'scope' above this line or read it in code that runs later.",
                "'scope' is read before it is declared, while the object is still being constructed: declare " +
                    "'scope' above this line or read it in code that runs later.",
                "'scope' is read before it is declared, while the object is still being constructed: declare " +
                    "'scope' above this line or read it in code that runs later.",
            ),
            rule.lintWithContext(environment, code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_reads_that_run_later_or_of_constants_WHEN_linted_THEN_reports_nothing() {
        val code = """
            class Scope

            object UserTable {
                val uuid = "uuid".padEnd(MAX_UUID_LENGTH)
                private const val MAX_UUID_LENGTH = 64
            }

            class BukkitCoreModule {
                val registrar by lazy { unconfinedScope }
                val reader = { counter }
                val reference = ::counter
                val listener = object : Runnable {
                    override fun run() = println(counter)
                }
                private val unconfinedScope = Scope()
                private var counter = 0

                fun mutate() {
                    counter++
                }

                val local: Int
                    get() = later

                val shadowed = run { val later = 1; later }
                private val later = 2
            }

            class Mapper {
                private val other = Mapper()
                val copied = other.later
                private val later = 1

                companion object {
                    val REF = CHAT_ID + 1
                    const val CHAT_ID = -1001L
                }
            }
        """.trimIndent()

        assertEquals(emptyList(), rule.lintWithContext(environment, code).map(Finding::message))
    }
}
