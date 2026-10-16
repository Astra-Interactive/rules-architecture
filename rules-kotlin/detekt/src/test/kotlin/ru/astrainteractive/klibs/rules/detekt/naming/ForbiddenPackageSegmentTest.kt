@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.naming

import dev.detekt.api.Config
import dev.detekt.test.TestConfig
import dev.detekt.test.lint
import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

internal class ForbiddenPackageSegmentTest {

    @Test
    fun GIVEN_package_with_layer_segments_WHEN_linted_THEN_reports_once_naming_each_segment() {
        val code = """
            package ru.astrainteractive.aspekt.domain.usecase

            class JailPlayer
        """.trimIndent()

        val findings = ForbiddenPackageSegment(Config.empty).lint(code)

        assertEquals(1, findings.size)
        assertTrue(findings.single().message.contains("'domain', 'usecase'"))
    }

    @Test
    fun GIVEN_package_with_roles_and_unit_segments_WHEN_linted_THEN_reports_nothing() {
        val code = """
            package com.flipperdevices.bsb.monetization.impl.data.revenuecat

            class RevenueCatEntitlementRepository
        """.trimIndent()

        assertEquals(0, ForbiddenPackageSegment(Config.empty).lint(code).size)
    }

    @Test
    fun GIVEN_configured_segments_WHEN_linted_THEN_uses_them_instead_of_defaults() {
        val code = """
            package com.flipperdevices.bsb.monetization.impl.domain

            class CheckEntitlement
        """.trimIndent()
        val rule = ForbiddenPackageSegment(TestConfig("forbiddenSegments" to listOf("impl")))

        val findings = rule.lint(code)

        assertEquals(1, findings.size)
        assertTrue(findings.single().message.contains("'impl'"))
    }
}
