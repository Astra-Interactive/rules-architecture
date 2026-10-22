@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class SealedFamilyFileTest {

    @Test
    fun GIVEN_variants_outside_the_sealed_file_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "com.example.order.model/OrderError.kt",
                "sealed class OrderError(message: String) : Exception(message)"
            )
            .main("com.example.order.model/OrderParseError.kt", "class OrderParseError : OrderError(\"parse\")")
            .main("com.example.order.model/RenameResult.kt", "sealed interface RenameResult<out T>")
            .main("com.example.order.model/Renamed.kt", "object Renamed : RenameResult<Nothing>")
            .load(UnitKind.CONTEXT)

        val messages = SealedFamilyFile().check(sources).map(Violation::message)

        assertEquals(
            listOf(
                "Move OrderParseError into OrderError.kt: a sealed hierarchy is one file with all its variants.",
                "Move Renamed into RenameResult.kt: a sealed hierarchy is one file with all its variants.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_sealed_hierarchies_in_one_file_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "com.example.order.model/OrderError.kt",
                """
                sealed class OrderError(message: String) : Exception(message) {
                    class Parse : OrderError("parse")
                }

                class OrderLoadError : OrderError("load")
                """
            )
            .main("com.example.order.model/Order.kt", "data class Order(val id: Int) : Comparable<Order>")
            .load(UnitKind.CONTEXT)

        assertEquals(emptyList(), SealedFamilyFile().check(sources))
    }
}
