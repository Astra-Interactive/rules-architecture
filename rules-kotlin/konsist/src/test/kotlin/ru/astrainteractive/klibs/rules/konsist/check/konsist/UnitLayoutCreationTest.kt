@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.konsist

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import kotlin.test.assertEquals

internal class UnitLayoutCreationTest {

    private fun rootOf(packages: List<String>, unitPath: String?): String {
        return UnitSources.createUnitLayout(
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
        val layout = UnitSources.createUnitLayout(
            listOf("x.jail.model"),
            rootPackage = "x",
            unitPath = ":jail",
            unitKind = UnitKind.CONTEXT
        )

        assertEquals("x", layout.rootPackage)
    }
}
