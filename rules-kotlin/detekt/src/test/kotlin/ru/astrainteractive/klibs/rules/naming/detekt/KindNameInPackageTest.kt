@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.naming.detekt

import dev.detekt.api.Config
import dev.detekt.core.config.YamlConfig
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import java.io.StringReader
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
    fun GIVEN_platform_service_subclass_in_root_package_WHEN_linted_THEN_reports_only_the_own_service() {
        val code = """
            package com.makeevrserg.empireprojekt.mobile.wear

            import android.app.Service
            import android.service.quicksettings.TileService

            interface StatusService

            class StatusesTileService : TileService()

            class DefaultStatusService : Service(), StatusService
        """.trimIndent()

        assertEquals(
            listOf("Move 'StatusService' to a 'service' package, or drop 'Service' from its name."),
            rule.lint(code).map { finding -> finding.message }
        )
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
    fun GIVEN_entity_suffix_in_model_below_room_WHEN_linted_THEN_reports_nothing() {
        val code = """
            package ru.astrainteractive.bsb.data.room.dictionary.model

            class DictionaryEntity
        """.trimIndent()

        assertEquals(0, rule.lint(code).size)
    }

    @Test
    fun GIVEN_model_suffix_in_model_below_data_without_technology_WHEN_linted_THEN_reports_it() {
        val code = """
            package ru.astrainteractive.astrarating.data.dao.rating.model

            data class RatingModel(val value: Int)
        """.trimIndent()

        assertEquals(1, rule.lint(code).size)
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

    @Test
    fun GIVEN_entity_suffix_below_a_project_technology_word_WHEN_linted_THEN_reports_only_without_the_word() {
        val code = """
            package x.data.mongo.rating.model

            class RatingEntity
        """.trimIndent()
        val projectRule = KindNameInPackage(YamlConfig.load(StringReader("technologyWords: [ mongo ]")))

        assertEquals(1, KindNameInPackage(Config.empty).lint(code).size)
        assertEquals(0, projectRule.lint(code).size)
    }
}
