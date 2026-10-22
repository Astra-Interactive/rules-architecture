@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.layout

import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class PlacementTest {

    @Test
    fun GIVEN_placements_WHEN_described_THEN_names_their_place() {
        val descriptions = listOf(
            Placement(emptyList(), null, null, null, emptyList()),
            Placement(listOf("rating", "players"), Kind.ENTRY_POINT, "component", null, emptyList()),
            Placement(listOf("rating"), Kind.IMPLEMENTATION, null, null, emptyList()),
            Placement(listOf("paywall"), Kind.ENTRY_POINT, "composable", "screen", emptyList()),
        ).map { placement -> placement.describe() }

        assertEquals(
            listOf("the unit root", "'rating/players/component'", "'rating'", "'paywall/composable/screen'"),
            descriptions
        )
    }
}
