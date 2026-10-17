@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.naming

import dev.detekt.api.Config
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class KindNameInPackageTest {
    private val rule = KindNameInPackage(Config.empty)

    @Test
    fun GIVEN_service_name_outside_service_package_WHEN_linted_THEN_reports_it() {
        val code = """
            package ru.astrainteractive.aspekt.module.jail

            class JailService
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_service_name_inside_service_package_WHEN_linted_THEN_reports_nothing() {
        val code = """
            package ru.astrainteractive.aspekt.module.jail.service

            class UnJailService
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_model_and_entity_suffix_in_domain_model_WHEN_linted_THEN_reports_each() {
        val code = """
            package ru.astrainteractive.aspekt.module.jail.model

            data class JailModel(val name: String)

            data class JailEntity(val name: String)
        """.trimIndent()

        assertEquals(2, rule.lint(code).size)
    }

    @Test
    fun GIVEN_entity_suffix_in_technology_model_WHEN_linted_THEN_reports_nothing() {
        val code = """
            package ru.astrainteractive.aspekt.module.jail.data.exposed.model

            class JailEntity
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_model_package_above_data_WHEN_linted_THEN_reports_suffix() {
        val code = """
            package ru.astrainteractive.aspekt.module.jail.model.data

            class JailModel
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
    }

    @Test
    fun GIVEN_suffix_outside_model_package_WHEN_linted_THEN_reports_nothing() {
        val code = """
            package ru.astrainteractive.aspekt.module.jail.viewmodel

            class JailViewModel

            val anonymous = object {}
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_helper_util_and_manager_classes_WHEN_linted_THEN_reports_each() {
        val code = """
            package ru.astrainteractive.astrarating.core.util

            class RatingHelper

            object StringUtils

            class CacheManager

            class Utility
        """.trimIndent()

        assertEquals(3, rule.lint(code).size)
    }
}
