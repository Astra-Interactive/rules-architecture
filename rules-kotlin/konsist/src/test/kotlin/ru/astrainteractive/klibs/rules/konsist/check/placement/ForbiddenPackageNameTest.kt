@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals
import kotlin.test.assertTrue

internal class ForbiddenPackageNameTest {

    @Test
    fun GIVEN_forbidden_words_below_the_root_WHEN_checked_THEN_reports_each_package_once(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.data.dao.rating.api",
                "RatingRepository.kt",
                "interface RatingRepository"
            )
            .main(
                "x.data.dao.rating.usecase",
                "RateUseCase.kt",
                "class RateUseCase"
            )
            .main(
                "x.data.dao.domain.repository",
                "One.kt",
                "class One"
            )
            .main(
                "x.data.dao.domain.repository",
                "Two.kt",
                "class Two"
            )
            .main(
                "x.data.dao.rating.table",
                "RatingTable.kt",
                "object RatingTable"
            )
            .main(
                "x.data.dao.rating.exposed.entity",
                "RatingEntity.kt",
                "class RatingEntity"
            )
            .main(
                "x.data.dao.rating.exposed.database",
                "RatingDatabase.kt",
                "class RatingDatabase"
            )
            .main(
                "x.data.dao.database",
                "Database.kt",
                "class Database"
            )
            .load(UnitKind.CONTEXT, unitPath = ":modules:data:dao")

        val violations = ForbiddenPackageName().check(sources)

        assertEquals(
            listOf(
                "x.data.dao.database",
                "x.data.dao.domain.repository",
                "x.data.dao.rating.table"
            ),
            violations.map(Violation::subject).sorted()
        )
        val expected = "Rename the package segments 'domain', 'repository'"
        assertTrue(violations.any { violation -> violation.message.startsWith(expected) })
    }

    @Test
    fun GIVEN_technology_unit_WHEN_checked_THEN_allows_table_entity_and_database(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.data.exposed.rating",
                "ExposedRatingRepository.kt",
                "class ExposedRatingRepository"
            )
            .main(
                "x.data.exposed.rating.table",
                "UserRatingTable.kt",
                "object UserRatingTable"
            )
            .main(
                "x.data.exposed.rating.entity",
                "UserRatingEntity.kt",
                "class UserRatingEntity"
            )
            .main(
                "x.data.exposed.database",
                "RatingDatabase.kt",
                "class RatingDatabase"
            )
            .load(UnitKind.TECHNOLOGY, unitPath = ":modules:data:exposed")

        assertEquals(emptyList(), ForbiddenPackageName().check(sources))
    }
}
