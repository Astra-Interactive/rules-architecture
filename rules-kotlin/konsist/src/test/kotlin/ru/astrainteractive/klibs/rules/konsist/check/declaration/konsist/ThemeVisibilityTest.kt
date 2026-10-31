@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class ThemeVisibilityTest {

    @Test
    fun GIVEN_feature_theme_objects_WHEN_checked_THEN_reports_each_not_internal(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.payment.impl.checkout.theme/CheckoutMotion.kt", "object CheckoutMotion")
            .main("x.payment.impl.checkout.theme/CheckoutSpacing.kt", "internal object CheckoutSpacing")
            .main("x.payment.impl.offer.theme/OfferMotion.kt", "private object OfferMotion")
            .load(UnitKind.CONTEXT, unitPath = ":payment:impl")

        assertEquals(
            listOf("CheckoutMotion", "OfferMotion"),
            ThemeVisibility().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_design_system_theme_objects_WHEN_checked_THEN_reports_each_hidden(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.core.ui.theme/AppTheme.kt", "object AppTheme")
            .main("x.core.ui.theme/AppSpacing.kt", "internal object AppSpacing")
            .load(UnitKind.CONTEXT, unitPath = ":core:ui")

        val violation = ThemeVisibility().check(sources).single()

        assertEquals("AppSpacing", violation.subject)
        assertEquals(
            "Make AppSpacing public: the design-system unit's theme is read by other units.",
            violation.message
        )
    }
}
