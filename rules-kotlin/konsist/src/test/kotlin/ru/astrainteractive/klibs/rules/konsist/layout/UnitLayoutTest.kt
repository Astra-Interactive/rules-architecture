@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.layout

import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

internal class UnitLayoutTest {
    private val packages = listOf(
        "com.example.jail",
        "com.example.jail.model",
        "com.example.jail.data.exposed.table",
        "com.example.jail.offer.model",
        "com.example.jail.presentation",
    )
    private val layout = UnitLayout.create(packages, rootPackage = null)

    @Test
    fun GIVEN_packages_of_a_context_WHEN_created_THEN_root_is_their_common_prefix() {
        assertEquals("com.example.jail", layout.rootPackage)
    }

    @Test
    fun GIVEN_packages_all_inside_roles_WHEN_created_THEN_trailing_role_segments_are_dropped() {
        val rolesOnly = UnitLayout.create(listOf("com.example.jail.model"), rootPackage = null)

        assertEquals("com.example.jail", rolesOnly.rootPackage)
    }

    @Test
    fun GIVEN_explicit_root_package_WHEN_created_THEN_uses_it() {
        val explicit = UnitLayout.create(packages, rootPackage = "com.example")

        assertEquals("com.example", explicit.rootPackage)
    }

    @Test
    fun GIVEN_no_packages_WHEN_created_THEN_root_is_the_default_package() {
        assertEquals("", UnitLayout.create(emptyList(), rootPackage = null).rootPackage)
    }

    @Test
    fun GIVEN_role_packages_WHEN_placed_THEN_resolves_role_and_technology() {
        assertEquals(
            Placement(module = null, role = null, subpackages = emptyList()),
            layout.placementOf("com.example.jail")
        )
        assertEquals(
            Placement(module = null, role = Role.DATA, subpackages = listOf("exposed", "table")),
            layout.placementOf("com.example.jail.data.exposed.table")
        )
    }

    @Test
    fun GIVEN_module_packages_WHEN_placed_THEN_resolves_module_and_role() {
        assertEquals(
            Placement(module = "offer", role = Role.MODEL, subpackages = emptyList()),
            layout.placementOf("com.example.jail.offer.model")
        )
        assertEquals(
            Placement(module = "offer", role = null, subpackages = emptyList()),
            layout.placementOf("com.example.jail.offer")
        )
    }

    @Test
    fun GIVEN_packages_outside_the_layout_WHEN_placed_THEN_has_no_placement() {
        assertNull(layout.placementOf("com.example.jail.presentation"))
        assertNull(layout.placementOf("com.example.jail.offer.presentation"))
        assertNull(layout.placementOf("com.example.jailhouse"))
        assertNull(layout.placementOf("org.other"))
    }
}
