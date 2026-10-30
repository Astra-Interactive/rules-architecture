@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.kdoc.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class KDocFillerTest {
    private val rule = KDocFiller(Config.empty)

    @Test
    fun GIVEN_filler_kdoc_WHEN_linted_THEN_reports_each_problem() {
        val code = """
            /**
             * This method is intended for retrieving a user by their identifier.
             *
             * @param userId the user id
             * @param name
             * @return the result
             */
            fun findUser(userId: Long, name: String): String = ""

            /**
             * Loads the order. It reads the cache. It falls back to the network. It retries twice! Is it slow?
             *
             * @return
             */
            fun loadOrder(): String = ""
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(
            listOf(
                "Delete the filler 'is intended for': say what the signature cannot, or write no KDoc.",
                "Delete the @return that restates the type, or say what is returned in the edge cases.",
                "Delete '@param userId' that repeats the name, or describe its meaning and constraints.",
                "Delete '@param name' that repeats the name, or describe its meaning and constraints.",
                "Delete the @return that restates the type, or say what is returned in the edge cases.",
                "Shorten the summary to at most 4 sentences: a longer one is a sign the declaration does too much.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_doc_bad_examples_WHEN_linted_THEN_reports_each_restatement() {
        val code = """
            /**
             * @param userId the identifier of the user
             * @return the user object
             */
            fun findUser(userId: Long): User

            /**
             * Function for calculating a discount.
             */
            fun calculateDiscount(price: BigDecimal, rate: Double): BigDecimal

            /**
             * @return the value
             */
            val name: String = ""
        """.trimIndent()

        assertEquals(
            listOf(
                "Delete the @return that restates the type, or say what is returned in the edge cases.",
                "Delete '@param userId' that repeats the name, or describe its meaning and constraints.",
                "Delete the filler 'Function for': say what the signature cannot, or write no KDoc.",
                "Delete the @return that restates the type, or say what is returned in the edge cases.",
            ),
            rule.lint(code).map(Finding::message)
        )
    }

    @Test
    fun GIVEN_kdoc_that_adds_information_WHEN_linted_THEN_reports_nothing() {
        val code = """
            /**
             * Calculates the discount amount, excluding promotional items.
             *
             * @param rate discount as a fraction in the range 0.0..1.0, not a percentage
             * @return amount rounded to cents using HALF_UP
             * @throws IllegalStateException never
             */
            fun calculateDiscount(price: Double, rate: Double): Double = price * rate

            /**
             * @return failure with [UserNotFoundError] for a deleted user as well as for an unknown id
             */
            fun findUser(userId: Long): Result<String> = Result.success("")

            /**
             * @return the cached user, or null after a logout
             */
            fun findCachedUser(): User?

            /**
             * @return the user
             */
            val user: User? = null
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }
}
