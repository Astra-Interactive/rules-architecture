@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class AndroidNamespaceTest {
    private fun sourcesWith(directory: File, namespace: String?): SourceTree {
        return SourceTree(directory, namespace)
            .main("com.flipperdevices.bsb.monetization.impl.offer.model/Plan.kt", "class Plan")
    }

    @Test
    fun GIVEN_namespace_other_than_the_root_package_WHEN_checked_THEN_reports_it(@TempDir directory: File) {
        val sources = sourcesWith(directory, "com.flipperdevices.monetization")
            .load(UnitKind.CONTEXT, rootPackage = "com.flipperdevices.bsb.monetization.impl")

        assertEquals(
            listOf(
                "Set the namespace of this Android unit to its root package " +
                    "com.flipperdevices.bsb.monetization.impl instead of com.flipperdevices.monetization."
            ),
            AndroidNamespace().check(sources).map(Violation::message)
        )
    }

    @Test
    fun GIVEN_matching_namespace_or_no_android_unit_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val matching = sourcesWith(File(directory, "matching"), "com.flipperdevices.bsb.monetization.impl")
            .load(UnitKind.CONTEXT, rootPackage = "com.flipperdevices.bsb.monetization.impl")
        val plain = sourcesWith(File(directory, "plain"), null)
            .load(UnitKind.CONTEXT, rootPackage = "com.flipperdevices.bsb.monetization.impl")

        assertEquals(emptyList(), AndroidNamespace().check(matching) + AndroidNamespace().check(plain))
    }
}
