@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.layout

import org.junit.jupiter.api.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

internal class UnitLayoutTest {

    private fun rootOf(packages: List<String>, unitPath: String?): String {
        return UnitLayout.create(
            packages,
            rootPackage = null,
            unitPath = unitPath,
            unitKind = UnitKind.CONTEXT
        )
            .rootPackage
    }

    @Test
    fun GIVEN_unit_paths_WHEN_root_detected_THEN_cuts_after_the_unit_name() {
        val event = rootOf(
            listOf("x.astrarating.event.kill.usecase", "x.astrarating.event.di"),
            ":modules:event-bukkit"
        )
        val papi = rootOf(
            listOf("x.astrarating.integration.papi.placeholder.expansion", "x.astrarating.integration.papi.di"),
            ":modules:integration-papi"
        )
        val dao = rootOf(
            listOf("x.astrarating.data.dao.rating.api", "x.astrarating.data.dao.rating.model"),
            ":modules:data:dao"
        )

        assertEquals(
            listOf(
                "x.astrarating.event",
                "x.astrarating.integration.papi",
                "x.astrarating.data.dao"
            ),
            listOf(
                event,
                papi,
                dao
            )
        )
    }

    @Test
    fun GIVEN_unit_name_absent_from_packages_WHEN_root_detected_THEN_uses_the_common_package() {
        assertEquals("x.astrarating", rootOf(listOf("x.astrarating", "x.astrarating.di"), ":instances:bukkit"))
        assertEquals("x.jail", rootOf(listOf("x.jail.model"), unitPath = null))
        assertEquals("x.jail", rootOf(listOf("x.jail"), unitPath = null))
        assertEquals("", rootOf(emptyList(), unitPath = null))
        assertEquals("", rootOf(listOf("a.b", "c.d"), unitPath = null))
    }

    @Test
    fun GIVEN_explicit_root_package_WHEN_created_THEN_uses_it() {
        val layout = UnitLayout.create(
            listOf("x.jail.model"),
            rootPackage = "x",
            unitPath = ":jail",
            unitKind = UnitKind.CONTEXT
        )

        assertEquals("x", layout.rootPackage)
    }

    @Test
    fun GIVEN_packages_of_a_unit_WHEN_placed_THEN_resolves_concepts_and_the_last_kind() {
        val layout = UnitLayout("x.gui.api", UnitKind.API)

        assertEquals(
            Placement(
                emptyList(),
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
                emptyList()
            ),
            layout.placementOf("x.gui.api.rating")
        )
        assertEquals(
            Placement(
                listOf("router"),
                Kind.CONTRACT,
                "api",
                emptyList()
            ),
            layout.placementOf("x.gui.api.router.api")
        )
        assertEquals(
            Placement(
                listOf("rating"),
                Kind.MODEL,
                "model",
                listOf("extra")
            ),
            layout.placementOf("x.gui.api.rating.model.extra")
        )
        assertNull(layout.placementOf("x.gui.apix"))
    }

    @Test
    fun GIVEN_technology_unit_WHEN_placed_THEN_concept_packages_hold_implementations() {
        val layout = UnitLayout("x.data.exposed", UnitKind.TECHNOLOGY)

        assertEquals(
            Placement(
                listOf("rating"),
                Kind.IMPLEMENTATION,
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
                emptyList()
            ),
            layout.placementOf("x.data.exposed")
        )
    }

    @Test
    fun GIVEN_default_root_package_WHEN_placed_THEN_every_package_is_below_it() {
        val layout = UnitLayout("", UnitKind.CONTEXT)

        assertEquals(
            Placement(
                listOf("jail"),
                Kind.MODEL,
                "model",
                emptyList()
            ),
            layout.placementOf("jail.model")
        )
    }
}
