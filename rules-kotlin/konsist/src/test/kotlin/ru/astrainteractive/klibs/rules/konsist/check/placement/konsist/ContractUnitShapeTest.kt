@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class ContractUnitShapeTest {

    @Test
    fun GIVEN_contract_unit_with_behaviour_and_plain_types_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.dictionary.contract/DictionaryResponse.kt",
                """
                import kotlinx.serialization.Serializable

                @Serializable
                data class DictionaryResponse(val id: Long) {
                    fun isEmpty(): Boolean = id == 0L
                }

                data class DictionaryRequest(val name: String)

                interface DictionaryRoute

                fun format(response: DictionaryResponse): String {
                    return response.id.toString()
                }
                """
            )
            .load(UnitKind.CONTRACT, rootPackage = "x.dictionary.contract")

        assertEquals(
            listOf(
                "DictionaryRequest",
                "DictionaryRoute",
                "format",
                "isEmpty"
            ),
            ContractUnitShape().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_contract_unit_of_wire_types_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.dictionary.contract/DictionaryRoutes.kt",
                """
                import kotlinx.serialization.Serializable

                object DictionaryRoutes {
                    const val ROOT = "/dictionary"

                    fun byId(id: Long): String = "${'$'}ROOT/${'$'}id"
                }

                enum class DictionaryKind { WORDS }

                @Serializable
                sealed interface DictionaryEvent {
                    fun describe(): String
                }
                """
            )
            .load(UnitKind.CONTRACT, rootPackage = "x.dictionary.contract")

        assertEquals(emptyList(), ContractUnitShape().check(sources))
    }
}
