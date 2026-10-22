@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.util

import dev.detekt.test.utils.compileContentForTest
import org.jetbrains.kotlin.psi.KtLambdaExpression
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class KtLambdaExpressionExtTest {

    @Test
    fun GIVEN_lambdas_in_several_positions_WHEN_their_call_is_found_THEN_names_the_call_or_nothing() {
        val file = compileContentForTest(
            """
            val trailing = runCatching { 1 }
            val inParentheses = launch({ 2 })
            val stored = { 3 }
            val annotated = Wrapper(@Marker { 4 })
            """.trimIndent()
        )

        val names = file.collectDescendantsOfType<KtLambdaExpression>().map(KtLambdaExpression::findCallName)

        assertEquals(
            listOf(
                "runCatching",
                "launch",
                null,
                null
            ),
            names
        )
    }
}
