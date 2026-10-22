@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class CoreShapeTest {

    private fun writeTree(directory: File): SourceTree {
        return SourceTree(directory)
            .main("x.core.rating.usecase/RatePlayerUseCase.kt", "class RatePlayerUseCase")
            .main("x.core.rating.api/RatingRepository.kt", "interface RatingRepository\n\ninterface Clock")
            .main("x.core.settings.model/Config.kt", "class Config")
    }

    @Test
    fun GIVEN_core_unit_with_use_case_and_repository_WHEN_checked_THEN_reports_both(@TempDir directory: File) {
        val sources = writeTree(directory).load(UnitKind.LIBRARY, unitPath = ":modules:core:api")

        assertEquals(
            listOf("RatePlayerUseCase", "RatingRepository"),
            CoreShape().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_feature_unit_or_unknown_path_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val feature = writeTree(directory).load(UnitKind.API, unitPath = ":modules:gui:api")
        val unknown = writeTree(directory).load(UnitKind.API)

        assertEquals(emptyList(), CoreShape().check(feature) + CoreShape().check(unknown))
    }
}
