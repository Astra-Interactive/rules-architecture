@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
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

    @Test
    fun GIVEN_same_named_nested_sealed_families_of_different_outer_types_WHEN_checked_THEN_reports_nothing(
        @TempDir directory: File
    ) {
        val family = """
            sealed interface Model {
                data object Loading : Model
                data object Error : Model
                class Loaded(val progress: Float) : Model
            }
        """.trimIndent()
        val sources = SourceTree(directory)
            .main("x.lesson.card.component/CardComponent.kt", "interface CardComponent {\n$family\n}")
            .main("x.lesson.comparison.component/ComparisonComponent.kt", "interface ComparisonComponent {\n$family\n}")
            .main(
                "x.tag.component/CreateTagComponent.kt",
                "interface CreateTagComponent { sealed interface Label { data object TagCreated : Label } }"
            )
            .main(
                "x.tag.component/ToggleTagComponent.kt",
                "interface ToggleTagComponent { sealed interface Label { data object OnChanged : Label } }"
            )
            .load(UnitKind.CONTEXT)

        assertEquals(emptyList(), SealedFamilyFile().check(sources))
    }

    @Test
    fun GIVEN_variants_naming_a_sealed_type_through_its_outer_type_or_an_import_WHEN_checked_THEN_reports_each(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.order.model/Order.kt", "interface Order { sealed interface State }")
            .main("x.order.model/Paid.kt", "class Paid : Order.State")
            .main("x.order.model/OrderError.kt", "sealed class OrderError(message: String) : Exception(message)")
            .main("x.order.model/Holder.kt", "class Holder { class Lost : OrderError(\"lost\") }")
            .main(
                "x.order.error.model/Timeout.kt",
                """
                import x.order.model.OrderError as Failure

                class Timeout : Failure("timeout")
            """
            )
            .load(UnitKind.CONTEXT)

        assertEquals(
            listOf(
                "Move Lost into OrderError.kt",
                "Move Paid into Order.kt",
                "Move Timeout into OrderError.kt",
            ),
            SealedFamilyFile().check(sources).map { violation -> violation.message.substringBefore(':') }.sorted()
        )
    }

    @Test
    fun GIVEN_nearer_or_imported_type_shadowing_a_sealed_name_WHEN_checked_THEN_reports_nothing(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.screen.model/Model.kt", "sealed interface Model")
            .main("x.screen.model/Screen.kt", "class Screen { interface Model\n class Loaded : Model }")
            .main("x.screen.model/Remote.kt", "import y.remote.Model\n\nclass Remote : Model")
            .load(UnitKind.CONTEXT)

        assertEquals(emptyList(), SealedFamilyFile().check(sources))
    }
}
