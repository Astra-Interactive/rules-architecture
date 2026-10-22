@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class NestedDataClassTest {

    @Test
    fun GIVEN_data_classes_with_one_owner_WHEN_checked_THEN_reports_them(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "com.example.jail.model",
                "Jail.kt",
                "data class Jail(val cell: Cell)"
            )
            .main(
                "com.example.jail.model",
                "Cell.kt",
                "data class Cell(val number: Int)"
            )
            .main(
                "com.example.jail.model",
                "Shop.kt",
                "class Shop {\n    val address: Address? = null\n}"
            )
            .main(
                "com.example.jail.model",
                "Address.kt",
                "data class Address(val street: String)"
            )
            .load(UnitKind.LIBRARY)

        val subjects = NestedDataClass().check(sources).map(Violation::subject)

        assertEquals(listOf("Address", "Cell"), subjects.sorted())
    }

    @Test
    fun GIVEN_data_classes_shared_or_returned_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "com.example.jail.model",
                "Price.kt",
                "data class Price(val amount: Long)"
            )
            .main(
                "com.example.jail.model",
                "Order.kt",
                "class Order(val price: Price)"
            )
            .main(
                "com.example.jail.model",
                "Invoice.kt",
                "class Invoice(val total: Price)"
            )
            .main(
                "com.example.jail.model",
                "Inmate.kt",
                "data class Inmate(val sentence: Sentence)"
            )
            .main(
                "com.example.jail.model",
                "Sentence.kt",
                "data class Sentence(val days: Int)"
            )
            .main(
                "com.example.jail",
                "FreeInmate.kt",
                "class FreeInmate { operator fun invoke(): Sentence = TODO() }"
            )
            .main(
                "com.example.jail.model",
                "Node.kt",
                "data class Node(val next: Node?)"
            )
            .main(
                "com.example.jail.model",
                "Builder.kt",
                "class Builder(seed: Seed)"
            )
            .main(
                "com.example.jail.model",
                "Seed.kt",
                "data class Seed(val value: Int)"
            )
            .load(UnitKind.CONTEXT)

        assertEquals(emptyList(), NestedDataClass().check(sources))
    }

    @Test
    fun GIVEN_sealed_variant_and_screen_state_with_one_owner_WHEN_checked_THEN_reports_nothing(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "com.example.splash.model",
                "SplashPaywallState.kt",
                """
                sealed interface SplashPaywallState

                data class Content(val title: String) : SplashPaywallState
                """
            )
            .main(
                "com.example.splash.model",
                "Holder.kt",
                "class Holder(val content: Content)"
            )
            .main(
                "com.example.splash.model",
                "RatingPlayersState.kt",
                "data class RatingPlayersState(val page: Int)"
            )
            .main(
                "com.example.splash.component",
                "RatingPlayersComponent.kt",
                "class RatingPlayersComponent(val state: RatingPlayersState)"
            )
            .load(UnitKind.CONTEXT)

        assertEquals(emptyList(), NestedDataClass().check(sources))
    }
}
