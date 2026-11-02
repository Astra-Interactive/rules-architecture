@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.extension.model

import org.gradle.testfixtures.ProjectBuilder
import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.core.unit.model.Checks
import ru.astrainteractive.klibs.rules.core.unit.model.RuleId
import ru.astrainteractive.klibs.rules.core.unit.model.UiComposition
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitRole
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

internal class LinterExtensionTest {
    private fun createExtension(): LinterExtension {
        return ProjectBuilder.builder().build().objects.newInstance(LinterExtension::class.java)
    }

    @Test
    fun GIVEN_equivalent_roles_WHEN_declared_twice_THEN_compares_normalized_configuration() {
        val extension = createExtension()
        extension.unit(UnitRole.ImplUi(composition = UiComposition.HOST))
        extension.unit(UnitRole.ImplUi(composition = UiComposition.HOST))
        extension.unitKind(UnitKind.IMPL_UI)

        assertEquals(UnitKind.IMPL_UI, extension.configuredUnitKind)
        assertEquals(UiComposition.HOST, extension.configuredUnitPolicy?.composition)
    }

    @Test
    fun GIVEN_same_kind_different_policy_WHEN_declared_twice_THEN_rejects_conflict() {
        val extension = createExtension()
        extension.unit(UnitRole.ImplUi())
        assertFailsWith<IllegalArgumentException> {
            extension.unit(UnitRole.ImplUi(composition = UiComposition.HOST))
        }
    }

    @Test
    fun GIVEN_legacy_kind_WHEN_new_role_disagrees_THEN_rejects_in_both_orders() {
        val first = createExtension()
        first.unitKind(UnitKind.API)
        assertFailsWith<IllegalArgumentException> { first.unit(UnitRole.Impl()) }
        val second = createExtension()
        second.unit(UnitRole.Api())
        assertFailsWith<IllegalArgumentException> { second.unitKind(UnitKind.IMPL) }
    }

    @Test
    fun GIVEN_mutable_checks_input_WHEN_caller_changes_map_THEN_keeps_policy_snapshot() {
        val reasons = mutableMapOf(RuleId.UNIT_README to "existing module")
        val extension = createExtension()
        extension.unit(UnitRole.Api(checks = Checks(disabled = reasons)))
        reasons.clear()

        assertEquals(mapOf("UnitReadme" to "existing module"), extension.configuredUnitPolicy?.checks?.disabledRules)
    }

    @Test
    fun GIVEN_empty_reason_WHEN_configured_THEN_fails_early() {
        assertFailsWith<IllegalArgumentException> { Checks(disabled = mapOf(RuleId.UNIT_README to " ")) }
    }
}
