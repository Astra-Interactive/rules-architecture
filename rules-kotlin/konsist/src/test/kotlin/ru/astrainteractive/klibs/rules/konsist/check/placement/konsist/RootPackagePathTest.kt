@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class RootPackagePathTest {

    @Test
    fun GIVEN_root_packages_following_their_unit_paths_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val event = SourceTree(File(directory, "event"))
            .main("x.astrarating.event.kill.event/KillListener.kt", "class KillListener")
            .load(UnitKind.CONTEXT, unitPath = ":modules:event-bukkit")
        val payment = SourceTree(File(directory, "payment"))
            .main("com.example.shop.payment.impl.offer.di/OfferModule.kt", "class OfferModule")
            .load(UnitKind.CONTEXT, unitPath = ":components:shop:payment:impl")

        assertEquals(
            emptyList(),
            RootPackagePath().check(event).plus(RootPackagePath().check(payment)).map(Violation::subject)
        )
    }

    @Test
    fun GIVEN_root_package_of_another_name_WHEN_checked_THEN_reports_it(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.astrarating.ratings.model/Rating.kt", "class Rating")
            .load(
                UnitKind.CONTEXT,
                rootPackage = "x.astrarating.ratings",
                unitPath = ":modules:gui:api"
            )

        val violation = RootPackagePath().check(sources).single()

        assertEquals(
            "Name the root package x.astrarating.ratings after the unit :modules:gui:api: it ends with gui.api, the " +
                "unit path in the project's convention.",
            violation.message
        )
    }

    @Test
    fun GIVEN_core_units_and_a_singular_package_WHEN_checked_THEN_reports_only_the_core_outside_core(
        @TempDir directory: File
    ) {
        val coreApi = SourceTree(File(directory, "core-api"))
            .main("x.astrarating.core.settings.model/Config.kt", "class Config")
            .load(
                UnitKind.CONTEXT,
                rootPackage = "x.astrarating.core",
                unitPath = ":modules:core:api"
            )
        val coreBukkit = SourceTree(File(directory, "core-bukkit"))
            .main("x.astrarating.core.di/BukkitModule.kt", "class BukkitModule")
            .load(
                UnitKind.CONTEXT,
                rootPackage = "x.astrarating.core",
                unitPath = ":modules:core:bukkit"
            )
        val advancement = SourceTree(File(directory, "advancement"))
            .main("x.aspekt.module.moneyadvancement.event/AdvancementEvent.kt", "class AdvancementEvent")
            .load(
                UnitKind.CONTEXT,
                rootPackage = "x.aspekt.module.moneyadvancement",
                unitPath = ":modules:money-advancements"
            )
        val outside = SourceTree(File(directory, "outside"))
            .main("x.aspekt.settings.model/Config.kt", "class Config")
            .load(
                UnitKind.CONTEXT,
                rootPackage = "x.aspekt",
                unitPath = ":modules:core:api"
            )

        val messages = listOf(coreApi, coreBukkit, advancement, outside)
            .flatMap { sources -> RootPackagePath().check(sources) }
            .map(Violation::message)

        assertEquals(
            listOf(
                "Name the root package x.aspekt after the unit :modules:core:api: it ends with core, the unit path " +
                    "in the project's convention."
            ),
            messages
        )
    }
}
