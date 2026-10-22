@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class ContractNameTest {

    @Test
    fun GIVEN_contracts_named_for_technology_or_generic_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.data.dao.rating.api/RatingDao.kt", "interface RatingDao")
            .main("x.data.dao.rating.api/UserStorage.kt", "interface UserStorage")
            .main(
                "x.data.dao.rating.api/RatingRepository.kt",
                "interface RatingRepository : CrudRepository<Rating, Long>"
            )
            .main("x.data.dao.rating.api/JailRepository.kt", "interface JailRepository : CrudOperations<Jail>")
            .load(UnitKind.CONTEXT, rootPackage = "x.data.dao")

        assertEquals(
            listOf(
                "Drop CrudOperations from JailRepository: a repository speaks the context's words and extends no " +
                    "generic base.",
                "Drop CrudRepository from RatingRepository: a repository speaks the context's words and extends no " +
                    "generic base.",
                "Rename RatingDao to <Aggregate>Repository: Dao names a technology, not the contract.",
                "Rename UserStorage to <Aggregate>Repository: Storage names a technology, not the contract.",
            ),
            ContractName().check(sources).map(Violation::message).sorted()
        )
    }

    @Test
    fun GIVEN_contracts_in_context_words_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.data.dao.rating.api/RatingRepository.kt", "interface RatingRepository : AutoCloseable")
            .main("x.data.dao.rating.api/RatingApi.kt", "interface RatingApi")
            .main("x.data.dao.rating.krate/SettingsKrate.kt", "interface SettingsKrate")
            .load(UnitKind.CONTEXT, rootPackage = "x.data.dao")

        assertEquals(emptyList(), ContractName().check(sources))
    }
}
