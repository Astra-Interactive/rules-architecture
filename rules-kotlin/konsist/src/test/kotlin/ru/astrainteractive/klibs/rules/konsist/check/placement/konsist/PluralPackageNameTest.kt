@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Vocabulary
import java.io.File
import kotlin.test.assertEquals

internal class PluralPackageNameTest {

    @Test
    fun GIVEN_plural_package_names_WHEN_checked_THEN_reports_each_package_once(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.gui.bukkit.ratings.model/Rating.kt", "data class Rating(val value: Int)")
            .main("x.gui.bukkit.ratings.model/RatingSort.kt", "enum class RatingSort { ASC }")
            .main("x.gui.bukkit.composable.components/CardComposable.kt", "fun Card() = Unit")
            .main("x.gui.bukkit.players.utils/PlayerExt.kt", "fun Int.double(): Int = this")
            .load(UnitKind.CONTEXT, rootPackage = "x.gui.bukkit")

        assertEquals(
            listOf(
                "Rename the package segments 'components' to the singular: a package is named like one concept or " +
                    "one kind.",
                "Rename the package segments 'players', 'utils' to the singular: a package is named like one " +
                    "concept or one kind.",
                "Rename the package segments 'ratings' to the singular: a package is named like one concept or one " +
                    "kind.",
            ),
            PluralPackageName().check(sources).map(Violation::message).sorted()
        )
    }

    @Test
    fun GIVEN_singular_and_inherently_plural_names_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.core.settings.model/AstraRatingConfig.kt", "class AstraRatingConfig")
            .main("x.core.news.model/News.kt", "class News")
            .main("x.core.status.model/Status.kt", "class Status")
            .main("x.core.analytics.api/AnalyticsRepository.kt", "interface AnalyticsRepository")
            .main("x.core.canvas.util/CanvasExt.kt", "fun Int.canvas(): Int = this")
            .main("x.core.address.model/Address.kt", "class Address")
            .main("x.core.analysis.model/Analysis.kt", "class Analysis")
            .main("other.outside.records/Record.kt", "class Record")
            .main("x.core.goods.model/Good.kt", "class Good")
            .load(
                UnitKind.CONTEXT,
                rootPackage = "x.core",
                vocabulary = Vocabulary(
                    entryPointWords = emptySet(),
                    technologyWords = emptySet(),
                    pluralConcepts = setOf("goods"),
                    platformClasses = emptySet()
                )
            )

        assertEquals(emptyList(), PluralPackageName().check(sources))
    }
}
