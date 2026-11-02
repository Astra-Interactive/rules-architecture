@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.core.unit.model

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.core.unit.util.toPolicy
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

internal class ChecksTest {
    @Test
    fun GIVEN_mutable_inputs_WHEN_checks_created_THEN_normalized_policy_is_a_snapshot() {
        val disabled = mutableMapOf(RuleId.UNIT_DEPENDENCY to "legacy module")
        val allowed = mutableMapOf(UnitReference(":feature:impl") to "temporary migration")
        val checks = Checks(disabled, allowed)
        disabled.clear()
        allowed.clear()

        assertEquals(
            CheckPolicy(
                mapOf("UnitDependency" to "legacy module"),
                mapOf(":feature:impl" to "temporary migration")
            ),
            checks.toPolicy()
        )
    }

    @Test
    fun GIVEN_invalid_reference_paths_WHEN_created_THEN_rejects_each_non_project_path() {
        listOf(
            "feature:impl",
            ":",
            ":feature::impl",
            ":feature:"
        ).forEach { path ->
            assertFailsWith<IllegalArgumentException>(path) { UnitReference(path) }
        }
    }

    @Test
    fun GIVEN_blank_reasons_WHEN_configured_THEN_rejects_each_exception() {
        assertFailsWith<IllegalArgumentException> {
            Checks(disabled = mapOf(RuleId.UNIT_DEPENDENCY to " "))
        }
        assertFailsWith<IllegalArgumentException> {
            Checks(allowedDependencies = mapOf(UnitReference(":feature:impl") to " "))
        }
        assertFailsWith<IllegalArgumentException> {
            CheckPolicy(mapOf("UnitDependency" to " "), emptyMap())
        }
        assertFailsWith<IllegalArgumentException> {
            CheckPolicy(emptyMap(), mapOf(":feature:impl" to " "))
        }
    }

    @Test
    fun GIVEN_unknown_rule_or_invalid_path_WHEN_policy_created_THEN_rejects_it() {
        val unknownRule = assertFailsWith<IllegalArgumentException> {
            CheckPolicy(mapOf("MissingRule" to "migration"), emptyMap())
        }
        val invalidPath = assertFailsWith<IllegalArgumentException> {
            CheckPolicy(emptyMap(), mapOf("feature:impl" to "migration"))
        }

        assertEquals(true, unknownRule.message?.contains("Unknown rule"))
        assertEquals(true, invalidPath.message?.contains("Gradle project path"))
    }
}
