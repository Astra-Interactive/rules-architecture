@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.layout

import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

internal class UnitLayoutTest {

    @Test
    fun GIVEN_packages_of_a_unit_WHEN_placed_THEN_resolves_concepts_and_the_last_kind() {
        val layout = UnitLayout("x.gui.api", UnitKind.API)

        assertEquals(Placement(emptyList(), null, null, null, emptyList()), layout.placementOf("x.gui.api"))
        assertEquals(Placement(listOf("rating"), null, null, null, emptyList()), layout.placementOf("x.gui.api.rating"))
        assertEquals(
            Placement(listOf("router"), Kind.CONTRACT, "api", null, emptyList()),
            layout.placementOf("x.gui.api.router.api")
        )
        assertEquals(
            Placement(listOf("rating"), Kind.MODEL, "model", null, listOf("extra")),
            layout.placementOf("x.gui.api.rating.model.extra")
        )
        assertNull(layout.placementOf("x.gui.apix"))
    }

    @Test
    fun GIVEN_technology_unit_WHEN_placed_THEN_concept_packages_hold_implementations() {
        val layout = UnitLayout("x.data.exposed", UnitKind.TECHNOLOGY)

        assertEquals(
            Placement(listOf("rating"), Kind.IMPLEMENTATION, null, null, emptyList()),
            layout.placementOf("x.data.exposed.rating")
        )
        assertEquals(Placement(emptyList(), null, null, null, emptyList()), layout.placementOf("x.data.exposed"))
    }

    @Test
    fun GIVEN_default_root_package_WHEN_placed_THEN_every_package_is_below_it() {
        val layout = UnitLayout("", UnitKind.CONTEXT)

        assertEquals(
            Placement(listOf("jail"), Kind.MODEL, "model", null, emptyList()),
            layout.placementOf("jail.model")
        )
    }

    @Test
    fun GIVEN_composable_sub_packages_WHEN_placed_THEN_composable_is_the_kind_and_the_segment_below_its_sub_kind() {
        val layout = UnitLayout("x.paywall.impl", UnitKind.CONTEXT)

        assertEquals(
            Placement(listOf("splash"), Kind.ENTRY_POINT, "composable", "screen", emptyList()),
            layout.placementOf("x.paywall.impl.splash.composable.screen")
        )
        assertEquals(
            Placement(emptyList(), Kind.ENTRY_POINT, "composable", "layout", listOf("frame")),
            layout.placementOf("x.paywall.impl.composable.layout.frame")
        )
        assertEquals(
            Placement(emptyList(), Kind.ENTRY_POINT, "composable", null, emptyList()),
            layout.placementOf("x.paywall.impl.composable")
        )
    }
}
