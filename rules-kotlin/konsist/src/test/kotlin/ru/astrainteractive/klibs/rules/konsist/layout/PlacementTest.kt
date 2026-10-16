@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.layout

import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

internal class PlacementTest {

    @Test
    fun GIVEN_each_role_WHEN_layer_asked_THEN_matches_the_direction_table() {
        val layers = listOf(
            Placement(module = null, role = null, subpackages = emptyList()),
            Placement(module = null, role = Role.COMMAND, subpackages = emptyList()),
            Placement(module = null, role = Role.MODEL, subpackages = emptyList()),
            Placement(module = null, role = Role.MAPPING, subpackages = emptyList()),
            Placement(module = null, role = Role.UTIL, subpackages = emptyList()),
            Placement(module = null, role = Role.DI, subpackages = emptyList()),
            Placement(module = null, role = Role.INTEGRATION, subpackages = emptyList()),
            Placement(module = null, role = Role.INTEGRATION, subpackages = listOf("vault")),
        ).map { placement -> placement.layer }

        assertEquals(
            listOf(
                Layer.USE_CASE,
                Layer.ENTRY_POINT,
                Layer.MODEL,
                Layer.MAPPING,
                Layer.UTIL,
                Layer.DI,
                Layer.CONTRACT,
                Layer.TECHNOLOGY,
            ),
            layers
        )
    }

    @Test
    fun GIVEN_subpackage_of_non_storage_role_WHEN_technology_asked_THEN_has_none() {
        assertNull(Placement(module = null, role = Role.COMMAND, subpackages = listOf("admin")).technology)
    }

    @Test
    fun GIVEN_placements_WHEN_described_THEN_names_their_place() {
        val descriptions = listOf(
            Placement(module = null, role = null, subpackages = emptyList()),
            Placement(module = "offer", role = null, subpackages = emptyList()),
            Placement(module = null, role = Role.DATA, subpackages = emptyList()),
            Placement(module = "offer", role = Role.DATA, subpackages = listOf("exposed", "table")),
        ).map { placement -> placement.describe() }

        assertEquals(
            listOf("the context root", "the root of module 'offer'", "the data contract", "'offer/data/exposed'"),
            descriptions
        )
    }
}
