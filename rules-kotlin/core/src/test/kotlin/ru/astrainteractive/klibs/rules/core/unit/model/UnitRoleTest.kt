@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.core.unit.model

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.core.unit.util.toPolicy
import kotlin.test.assertEquals

internal class UnitRoleTest {
    @Test
    fun GIVEN_all_role_variants_WHEN_normalized_THEN_keep_their_stable_kinds() {
        val roles = listOf(
            UnitRole.Context(),
            UnitRole.Api(),
            UnitRole.Contract(),
            UnitRole.EntryPoint(),
            UnitRole.Technology(),
            UnitRole.Fake(),
            UnitRole.Impl(),
            UnitRole.Platform(),
            UnitRole.CoreApi(),
            UnitRole.CorePlatform(),
            UnitRole.CoreUi(),
            UnitRole.StorageContract(),
            UnitRole.ImplUi(),
            UnitRole.Core()
        )

        assertEquals(UnitKind.entries, roles.map(UnitRole::kind))
    }

    @Test
    fun GIVEN_impl_ui_defaults_WHEN_normalized_THEN_use_local_sibling_policy() {
        assertEquals(
            UnitPolicy(
                UiComposition.LOCAL,
                ImplementationAccess.SIBLINGS,
                CheckPolicy(emptyMap(), emptyMap())
            ),
            UnitRole.ImplUi().toPolicy()
        )
    }

    @Test
    fun GIVEN_legacy_and_core_roles_WHEN_normalized_THEN_do_not_gain_implementation_access() {
        val kinds = listOf(
            UnitRole.Context(),
            UnitRole.Impl(),
            UnitRole.Core()
        )

        kinds.forEach { role ->
            assertEquals(ImplementationAccess.NONE, role.toPolicy().implementationAccess)
        }
    }

    @Test
    fun GIVEN_custom_impl_ui_policy_WHEN_normalized_THEN_preserves_host_and_no_sibling_access() {
        val policy = UnitRole.ImplUi(
            composition = UiComposition.HOST,
            implementationAccess = ImplementationAccess.NONE
        ).toPolicy()

        assertEquals(UiComposition.HOST, policy.composition)
        assertEquals(ImplementationAccess.NONE, policy.implementationAccess)
    }
}
