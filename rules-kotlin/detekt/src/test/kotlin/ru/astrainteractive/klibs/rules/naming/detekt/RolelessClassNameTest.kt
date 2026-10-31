@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.naming.detekt

import dev.detekt.api.Config
import dev.detekt.api.Finding
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class RolelessClassNameTest {
    private val rule = RolelessClassName(Config.empty)

    @Test
    fun GIVEN_roleless_and_misplaced_names_WHEN_linted_THEN_reports_each() {
        val code = """
            package com.example.rating.model

            class OrderValidator

            interface RepositoryFactory

            class RatingCacheWarmUp

            object ConfigSnakeCaseMigration

            class SplashViewModel
        """.trimIndent()

        val messages = rule.lint(code).map(Finding::message)

        assertEquals(
            listOf(
                "Remove 'OrderValidator': a validator duplicates a model invariant; validate the input at the " +
                    "entry point, guard the invariant in the value object or the aggregate's rules in util.",
                "Remove 'RepositoryFactory': dependencies are constructed in di/, and a factory with a rule is a " +
                    "top-level create<Aggregate>() in util.",
                "Move 'RatingCacheWarmUp' to a 'warmup' package: the WarmUp suffix names that kind.",
                "Move 'ConfigSnakeCaseMigration' to a 'migration' package: the Migration suffix names that kind.",
                "Move 'SplashViewModel' to a 'viewmodel' package: the ViewModel suffix names that kind.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_kind_names_in_their_packages_WHEN_linted_THEN_reports_nothing() {
        val warmUp = """
            package com.example.data.exposed.rating.warmup

            class RatingCacheWarmUp
        """.trimIndent()
        val migration = """
            package com.example.core.settings.migration

            class ConfigSnakeCaseMigration
        """.trimIndent()
        val viewModel = """
            package com.example.splash.viewmodel

            class SplashViewModel

            class Validation

            interface CheckoutComponent {
                fun interface Factory {
                    operator fun invoke(onPaidCallback: OnPaidCallback): CheckoutComponent
                }
            }
        """.trimIndent()

        val findings = listOf(
            warmUp,
            migration,
            viewModel
        ).sumOf { code -> rule.lint(code).size }

        assertEquals(0, findings)
    }

    @Test
    fun GIVEN_type_named_like_its_supertype_WHEN_linted_THEN_reports_only_the_own_factory() {
        val code = """
            package com.example.towns.paging

            data class TownyPageContext(val page: Int) : PageContext {
                object Factory : PageContext.Factory<TownyPageContext> {
                    override fun next(pageContext: TownyPageContext) = pageContext.copy(page = pageContext.page + 1)
                }
            }

            class TownsFactory : PageContext.Factory<TownyPageContext>
        """.trimIndent()

        assertEquals(
            listOf("TownsFactory"),
            rule.lint(code).map { finding -> finding.message.substringAfter("'").substringBefore("'") }
        )
    }
}
