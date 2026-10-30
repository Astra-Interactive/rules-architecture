@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class ComposeKindWithoutComposeTest {

    @Test
    fun GIVEN_compose_words_in_a_unit_without_compose_WHEN_checked_THEN_reports_each_package_but_viewmodel(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.gui.rating.viewmodel/RatingViewModel.kt", "class RatingViewModel")
            .main("x.gui.rating.viewmodel/RatingState.kt", "class RatingState")
            .main("x.gui.theme/Colors.kt", "object Colors")
            .main("x.gui.rating.composable/RatingView.kt", "fun ratingView() = Unit")
            .main("x.gui.rating.model/Rating.kt", "import java.io.File\n\nclass Rating")
            .load(UnitKind.CONTEXT, rootPackage = "x.gui")

        assertEquals(
            listOf("x.gui.rating.composable", "x.gui.theme"),
            ComposeKindWithoutCompose().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_compose_words_in_a_compose_unit_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.gui.rating.composable/RatingComposable.kt", "import androidx.compose.runtime.Composable")
            .load(UnitKind.CONTEXT, rootPackage = "x.gui")

        assertEquals(emptyList(), ComposeKindWithoutCompose().check(sources))
    }
}
