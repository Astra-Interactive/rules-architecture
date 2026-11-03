@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.layout.model

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import kotlin.test.assertEquals
import kotlin.test.assertNull

internal class UnitLayoutTest {

    @Test
    fun GIVEN_platform_and_technology_roles_WHEN_placing_concepts_THEN_accepts_implementations_but_not_root_files() {
        listOf(
            UnitKind.CORE_PLATFORM,
            UnitKind.PLATFORM,
            UnitKind.TECHNOLOGY
        ).forEach { role ->
            val layout = UnitLayout(
                "x.core",
                role,
                SourceTree.NO_PROJECT_WORDS
            )

            assertEquals(
                Kind.IMPLEMENTATION,
                layout.placementOf("x.core.item")?.kind,
                role.name
            )
            assertEquals(
                Kind.IMPLEMENTATION,
                layout.placementOf("x.core.economy.currency")?.kind,
                role.name
            )
            assertNull(layout.placementOf("x.core")?.kind, role.name)
            assertNull(layout.placementOf("x.other.item"), role.name)
        }
    }

    @Test
    fun GIVEN_platform_core_WHEN_placing_explicit_kinds_THEN_keeps_their_restrictions() {
        val layout = UnitLayout(
            "x.core",
            UnitKind.CORE_PLATFORM,
            SourceTree.NO_PROJECT_WORDS
        )
        val kinds = mapOf(
            "item.api" to Kind.CONTRACT,
            "item.model" to Kind.MODEL,
            "item.util" to Kind.UTIL,
            "format.di" to Kind.DI,
            "item.usecase" to Kind.USECASE,
            "item.menu" to Kind.ENTRY_POINT
        )

        kinds.forEach { (path, kind) ->
            assertEquals(
                kind,
                layout.placementOf("x.core.$path")?.kind,
                path
            )
        }
    }

    @Test
    fun GIVEN_other_unit_roles_WHEN_placing_loose_concepts_THEN_still_requires_a_kind() {
        val roles = UnitKind.entries - setOf(
            UnitKind.CORE_PLATFORM,
            UnitKind.PLATFORM,
            UnitKind.TECHNOLOGY
        )

        roles.forEach { role ->
            val layout = UnitLayout(
                "x.core",
                role,
                SourceTree.NO_PROJECT_WORDS
            )

            assertNull(layout.placementOf("x.core.item")?.kind, role.name)
        }
    }

    @Test
    fun GIVEN_packages_of_a_unit_WHEN_placed_THEN_resolves_concepts_and_the_last_kind() {
        val layout = UnitLayout(
            "x.gui.api",
            UnitKind.API,
            SourceTree.NO_PROJECT_WORDS
        )

        assertEquals(
            Placement(
                emptyList(),
                null,
                null,
                null,
                emptyList()
            ),
            layout.placementOf("x.gui.api")
        )
        assertEquals(
            Placement(
                listOf("rating"),
                null,
                null,
                null,
                emptyList()
            ),
            layout.placementOf("x.gui.api.rating")
        )
        assertEquals(
            Placement(
                listOf("router"),
                Kind.CONTRACT,
                "api",
                null,
                emptyList()
            ),
            layout.placementOf("x.gui.api.router.api")
        )
        assertEquals(
            Placement(
                listOf("rating"),
                Kind.MODEL,
                "model",
                null,
                listOf("extra")
            ),
            layout.placementOf("x.gui.api.rating.model.extra")
        )
        assertNull(layout.placementOf("x.gui.apix"))
    }

    @Test
    fun GIVEN_technology_unit_WHEN_placed_THEN_concept_packages_hold_implementations() {
        val layout = UnitLayout(
            "x.data.exposed",
            UnitKind.TECHNOLOGY,
            SourceTree.NO_PROJECT_WORDS
        )

        assertEquals(
            Placement(
                listOf("rating"),
                Kind.IMPLEMENTATION,
                null,
                null,
                emptyList()
            ),
            layout.placementOf("x.data.exposed.rating")
        )
        assertEquals(
            Placement(
                emptyList(),
                null,
                null,
                null,
                emptyList()
            ),
            layout.placementOf("x.data.exposed")
        )
    }

    @Test
    fun GIVEN_default_root_package_WHEN_placed_THEN_every_package_is_below_it() {
        val layout = UnitLayout(
            "",
            UnitKind.CONTEXT,
            SourceTree.NO_PROJECT_WORDS
        )

        assertEquals(
            Placement(
                listOf("jail"),
                Kind.MODEL,
                "model",
                null,
                emptyList()
            ),
            layout.placementOf("jail.model")
        )
    }

    @Test
    fun GIVEN_composable_sub_packages_WHEN_placed_THEN_composable_is_the_kind_and_the_segment_below_its_sub_kind() {
        val layout = UnitLayout(
            "x.paywall.impl",
            UnitKind.CONTEXT,
            SourceTree.NO_PROJECT_WORDS
        )

        assertEquals(
            Placement(
                listOf("splash"),
                Kind.ENTRY_POINT,
                "composable",
                "screen",
                emptyList()
            ),
            layout.placementOf("x.paywall.impl.splash.composable.screen")
        )
        assertEquals(
            Placement(
                emptyList(),
                Kind.ENTRY_POINT,
                "composable",
                "layout",
                listOf("frame")
            ),
            layout.placementOf("x.paywall.impl.composable.layout.frame")
        )
        assertEquals(
            Placement(
                emptyList(),
                Kind.ENTRY_POINT,
                "composable",
                null,
                emptyList()
            ),
            layout.placementOf("x.paywall.impl.composable")
        )
    }
}
