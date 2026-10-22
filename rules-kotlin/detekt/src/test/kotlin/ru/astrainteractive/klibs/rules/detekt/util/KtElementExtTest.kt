@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.util

import dev.detekt.test.utils.compileContentForTest
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

internal class KtElementExtTest {

    @Test
    fun GIVEN_properties_in_class_and_file_WHEN_found_from_a_call_THEN_the_innermost_wins() {
        val file = compileContentForTest(
            """
            val state = 0
            val global = 1

            class Holder {
                val state = 2

                fun read() = println(state)
            }
            """.trimIndent()
        )
        val call = file.collectDescendantsOfType<KtCallExpression>().single()

        assertEquals("2", call.findEnclosingProperty("state")?.initializer?.text)
        assertEquals("1", call.findEnclosingProperty("global")?.initializer?.text)
        assertNull(call.findEnclosingProperty("missing"))
    }
}
