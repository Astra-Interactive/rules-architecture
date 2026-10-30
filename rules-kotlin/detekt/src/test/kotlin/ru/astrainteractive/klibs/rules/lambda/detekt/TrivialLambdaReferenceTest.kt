@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.lambda.detekt

import dev.detekt.api.Config
import dev.detekt.test.lintWithContext
import dev.detekt.test.utils.createEnvironment
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class TrivialLambdaReferenceTest {
    private val rule = TrivialLambdaReference(Config.empty)
    private val environment = createEnvironment()

    @Test
    fun GIVEN_lambdas_that_only_pass_their_parameters_WHEN_linted_THEN_reports_each_with_its_reference() {
        val code = """
            fun render(ids: List<Long>, users: List<User>, repository: Repository, viewModel: PaywallViewModel) {
                ids.map { id -> UserId(id) }
                ids.map { id -> format(id) }
                users.map { user -> user.toContent() }
                users.map { user -> user.name }
                users.forEach { user -> repository.save(user) }
                users.fold(0) { total, user -> total.plus(user) }
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
}
