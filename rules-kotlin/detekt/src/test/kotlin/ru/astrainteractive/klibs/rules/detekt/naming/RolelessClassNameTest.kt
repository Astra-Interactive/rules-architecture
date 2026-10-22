@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.naming

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
        """.trimIndent()

        val findings = listOf(
            warmUp,
            migration,
            viewModel
        ).sumOf { code -> rule.lint(code).size }

        assertEquals(0, findings)
    }
}
