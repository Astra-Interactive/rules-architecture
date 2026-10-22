@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.layout

import org.junit.jupiter.api.Test
import kotlin.test.assertEquals

internal class PlacementTest {

    @Test
    fun GIVEN_placements_WHEN_described_THEN_names_their_place() {
        val descriptions = listOf(
            Placement(
                emptyList(),
                null,
                null,
                emptyList()
            ),
            Placement(
                listOf("rating", "players"),
                Kind.ENTRY_POINT,
                "component",
                emptyList()
            ),
            Placement(
                listOf("rating"),
                Kind.IMPLEMENTATION,
                null,
                emptyList()
            ),
        ).map(Placement::describe)

        assertEquals(
            listOf(
                "the unit root",
                "'rating/players/component'",
                "'rating'"
            ),
            descriptions
        )
    }
}
