@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.naming.detekt

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

internal class KindNameInPackageTest {
    private val rule = KindNameInPackage(Config.empty)
    private val environment = createEnvironment()

    @Test
    fun GIVEN_service_word_without_platform_role_WHEN_linted_THEN_reports_nothing() {
        val code = """
            package ru.astrainteractive.aspekt.module.jail

            class JailService
            data class ServiceCategoryItem(val name: String)
            object ServicesPreviewFixture
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_service_name_inside_service_package_WHEN_linted_THEN_reports_nothing() {
        val code = """
            package ru.astrainteractive.aspekt.module.jail.service

            class UnJailService
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_resolved_platform_service_hierarchy_WHEN_linted_THEN_reports_role_even_without_service_in_name() {
        val code = """
            package example.worker

            import android.app.Service as AndroidWorker

            interface StatusService
            open class BaseWorker : AndroidWorker()
            class SyncWorker : BaseWorker(), StatusService
        """.trimIndent()
        val platform = "package android.app\nopen class Service"

        val messages = rule.lintWithContext(
            environment,
            code,
            platform
        ).map { finding -> finding.message }

        assertEquals(2, messages.size)
        assertTrue(messages.all { message -> message.contains("subtype of android.app.Service") })
        assertTrue(messages.all { message -> message.contains("known types and annotations only") })
    }

    @Test
    fun GIVEN_framework_annotation_and_custom_service_types_WHEN_configured_THEN_recognizes_them() {
        val code = """
            package example.worker

            import com.intellij.openapi.components.Service as PlatformComponent
            import custom.Managed

            @PlatformComponent
            class PreviewOwner
            @Managed
            class CustomWorker
            class CustomTimer : custom.Timer()
        """.trimIndent()
        val platform = "package com.intellij.openapi.components\nannotation class Service"
        val custom = "package custom\nannotation class Managed\nopen class Timer"
        val settings = "serviceBaseTypes: [ custom.Timer ]\nserviceAnnotations: [ custom.Managed ]"
        val configured = KindNameInPackage(YamlConfig.load(StringReader(settings)))

        assertEquals(
            1,
            rule.lintWithContext(
                environment,
                code,
                platform,
                custom
            ).size
        )
        assertEquals(
            3,
            configured.lintWithContext(
                environment,
                code,
                platform,
                custom
            ).size
        )
    }

    @Test
    fun GIVEN_recognized_service_in_service_package_WHEN_linted_THEN_reports_nothing() {
        val code = "package example.service\nclass Worker : android.app.Service()"
        val platform = "package android.app\nopen class Service"

        assertEquals(
            0,
            rule.lintWithContext(
                environment,
                code,
                platform
            ).size
        )
    }

    @Test
    fun GIVEN_platform_service_in_entry_point_WHEN_linted_THEN_reports_nothing() {
        val context = UnitContext(
            UnitKind.ENTRY_POINT,
            ProductKind.APP,
            false,
            emptySet(),
            emptyList()
        )
        val config = YamlConfig.load(StringReader("unitContext: '${context.encode()}'"))
        val code = "package example.worker\nclass Worker : android.app.Service()"
        val platform = "package android.app\nopen class Service"
        val findings = KindNameInPackage(config).lintWithContext(
            environment,
            code,
            platform
        )

        assertEquals(0, findings.size)
    }

    @Test
    fun GIVEN_model_and_entity_suffix_in_domain_model_WHEN_linted_THEN_reports_each() {
        val code = """
            package ru.astrainteractive.aspekt.module.jail.model

            data class JailModel(val name: String)

            data class JailEntity(val name: String)
        """.trimIndent()

        assertEquals(2, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_entity_suffix_in_technology_model_WHEN_linted_THEN_reports_nothing() {
        val code = """
            package ru.astrainteractive.aspekt.module.jail.data.exposed.model

            class JailEntity
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_entity_suffix_in_model_below_room_WHEN_linted_THEN_reports_nothing() {
        val code = """
            package ru.astrainteractive.bsb.data.room.dictionary.model

            class DictionaryEntity
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_model_suffix_in_model_below_data_without_technology_WHEN_linted_THEN_reports_it() {
        val code = """
            package ru.astrainteractive.astrarating.data.dao.rating.model

            data class RatingModel(val value: Int)
        """.trimIndent()

        assertEquals(1, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_model_package_above_data_WHEN_linted_THEN_reports_suffix() {
        val code = """
            package ru.astrainteractive.aspekt.module.jail.model.data

            class JailModel
        """.trimIndent()

        assertEquals(1, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_suffix_outside_model_package_WHEN_linted_THEN_reports_nothing() {
        val code = """
            package ru.astrainteractive.aspekt.module.jail.viewmodel

            class JailViewModel

            val anonymous = object {}
        """.trimIndent()

        assertEquals(0, rule.lintWithContext(environment, code).size)
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

        assertEquals(3, rule.lintWithContext(environment, code).size)
    }

    @Test
    fun GIVEN_entity_suffix_below_a_project_technology_word_WHEN_linted_THEN_reports_only_without_the_word() {
        val code = """
            package x.data.mongo.rating.model

            class RatingEntity
        """.trimIndent()
        val projectRule = KindNameInPackage(YamlConfig.load(StringReader("technologyWords: [ mongo ]")))

        assertEquals(1, KindNameInPackage(Config.empty).lintWithContext(environment, code).size)
        assertEquals(0, projectRule.lintWithContext(environment, code).size)
    }
}
