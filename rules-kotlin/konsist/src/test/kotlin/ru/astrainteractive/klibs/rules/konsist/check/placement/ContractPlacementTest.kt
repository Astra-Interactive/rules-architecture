@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class ContractPlacementTest {

    @Test
    fun GIVEN_contracts_and_implementations_out_of_place_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.data.dao.rating.api", "RatingRepository.kt", "interface RatingRepository")
            .main(
                "x.data.dao.rating.api",
                "CachedRatingRepository.kt",
                "class CachedRatingRepository : RatingRepository"
            )
            .main(
                "x.data.dao.rating.memory",
                "InMemoryRatingRepository.kt",
                "class InMemoryRatingRepository : RatingRepository"
            )
            .main("x.data.dao.rating.model", "UserRepository.kt", "interface UserRepository")
            .main("x.data.dao.rating.remote", "HttpRating.kt", "class HttpRating")
            .main("x.data.dao.rating.remote", "HttpUser.kt", "class HttpUser")
            .load(UnitKind.CONTEXT, unitPath = ":modules:data:dao")

        val subjects = ContractPlacement().check(sources).map { violation -> violation.subject }.sorted()

        assertEquals(listOf("CachedRatingRepository", "UserRepository", "x.data.dao.rating.remote"), subjects)
    }
}
