@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureChecker
import ru.astrainteractive.klibs.rules.konsist.check.konsist.SuppressionPolicy
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class ContractNameTest {

    @Test
    fun GIVEN_contracts_named_for_technology_or_generic_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.data.dao.rating.api/RatingDao.kt", "interface RatingDao")
            .main("x.data.dao.rating.api/UserStorage.kt", "interface UserStorage")
            .main("x.data.dao.rating.api/MonetizationApi.kt", "interface MonetizationApi")
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
                "Rename MonetizationApi to <Aggregate>Repository: Api is not a name for the contract.",
                "Rename RatingDao to <Aggregate>Repository: Dao is not a name for the contract.",
                "Rename UserStorage to <Aggregate>Repository: Storage is not a name for the contract.",
            ),
            ContractName().check(sources).map(Violation::message).sorted()
        )
    }

    @Test
    fun GIVEN_contracts_in_context_words_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.data.dao.rating.api/RatingRepository.kt", "interface RatingRepository : AutoCloseable")
            .main("x.data.dao.rating.api/GuiRouter.kt", "interface GuiRouter")
            .main("x.data.dao.rating.krate/SettingsKrate.kt", "interface SettingsKrate")
            .load(UnitKind.CONTEXT, rootPackage = "x.data.dao")

        assertEquals(emptyList(), ContractName().check(sources))
    }

    @Test
    fun GIVEN_suppressed_contract_WHEN_checked_THEN_reports_only_unsuppressed_contract(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.data.dao.rating.api/RatingDao.kt",
                """
                @Suppress("ContractName")
                interface RatingDao
                """
            )
            .main("x.data.dao.rating.api/UserStorage.kt", "interface UserStorage")
            .load(UnitKind.CONTEXT, rootPackage = "x.data.dao")
        val checker = ArchitectureChecker(listOf(ContractName()), SuppressionPolicy())

        assertEquals(listOf("UserStorage"), checker.check(sources).map(Violation::subject))
    }
}
