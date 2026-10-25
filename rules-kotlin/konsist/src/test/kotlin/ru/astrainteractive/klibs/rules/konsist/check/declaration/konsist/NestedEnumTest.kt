@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class NestedEnumTest {

    @Test
    fun GIVEN_enum_used_through_one_type_WHEN_checked_THEN_reports_it(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("com.example.paywall.model/FadeSide.kt", "enum class FadeSide { TOP, BOTTOM }")
            .main("com.example.paywall.model/SoftEdge.kt", "data class SoftEdge(val sides: List<FadeSide>)")
            .load(UnitKind.CONTEXT)

        assertEquals(
            listOf("Nest FadeSide in SoftEdge: an enum used only through one type is nested in it."),
            NestedEnum().check(sources).map(Violation::message)
        )
    }

    @Test
    fun GIVEN_enums_shared_or_named_elsewhere_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("com.example.paywall.model/Stage.kt", "enum class Stage { OFFER }")
            .main("com.example.paywall.model/Card.kt", "data class Card(val stage: Stage)")
            .main("com.example.paywall.model/Banner.kt", "data class Banner(val stage: Stage)")
            .main("com.example.paywall.model/Size.kt", "enum class Size { SMALL }")
            .main("com.example.paywall.model/Button.kt", "data class Button(val size: Size)")
            .main("com.example.paywall.util/SizeExt.kt", "fun selectSize(): Size = Size.SMALL")
            .main("com.example.paywall.model/Unused.kt", "enum class Unused { ONE }\n\nclass Plain")
            .load(UnitKind.CONTEXT)

        assertEquals(emptyList(), NestedEnum().check(sources))
    }
}
