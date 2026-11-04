@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File
import kotlin.test.assertEquals

internal class ContractPlacementTest {

    @Test
    fun GIVEN_platform_core_contract_and_sealed_adapters_WHEN_checked_THEN_only_contract_needs_api(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.core.item/ItemProvider.kt", "interface ItemProvider")
            .main("x.core.item/SpigotShopItemStack.kt", "sealed interface SpigotShopItemStack")
            .main("x.core.item/SpigotTitleItemStack.kt", "sealed interface SpigotTitleItemStack")
            .main("x.core.item.api/ShopItemStack.kt", "interface ShopItemStack")
            .load(
                UnitKind.CORE_PLATFORM,
                rootPackage = "x.core",
                unitPath = ":modules:core:bukkit"
            )

        assertEquals(listOf("ItemProvider"), ContractPlacement().check(sources).map(Violation::subject))
    }

    @Test
    fun GIVEN_contracts_and_implementations_out_of_place_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.data.dao.rating.api",
                "RatingRepository.kt",
                "interface RatingRepository"
            )
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
            .main(
                "x.data.dao.rating.model",
                "UserRepository.kt",
                "interface UserRepository"
            )
            .main(
                "x.data.dao.rating.remote",
                "HttpRating.kt",
                "class HttpRating"
            )
            .main(
                "x.data.dao.rating.remote",
                "HttpUser.kt",
                "class HttpUser"
            )
            .load(UnitKind.CONTEXT, unitPath = ":modules:data:dao")

        val subjects = ContractPlacement().check(sources).map(Violation::subject).sorted()

        assertEquals(
            listOf(
                "CachedRatingRepository",
                "UserRepository",
                "x.data.dao.rating.remote"
            ),
            subjects
        )
    }

    @Test
    fun GIVEN_interfaces_across_kinds_WHEN_checked_THEN_reports_each_outside_their_homes(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.offer.api/OfferApi.kt", "interface OfferGateway")
            .main("x.offer.usecase/LoadOfferUseCase.kt", "interface LoadOfferUseCase")
            .main("x.offer.component/OfferComponent.kt", "interface OfferComponent")
            .main("x.offer.viewmodel/OfferViewModel.kt", "interface OfferViewModel")
            .main("x.di/OfferModule.kt", "interface OfferModule")
            .main("x.offer.model/OfferResult.kt", "sealed interface OfferResult")
            .main("x.offer.model/Priced.kt", "interface Priced")
            .main("x.offer.ktor/Http.kt", "fun interface HttpEngine")
            .load(UnitKind.CONTEXT, rootPackage = "x")

        val messages = ContractPlacement().check(sources).map(Violation::message).sorted()

        assertEquals(
            listOf(
                "Move HttpEngine to the api package of its concept: a contract sits in api.",
                "Move Priced to the api package of its concept: a contract sits in api.",
            ),
            messages
        )
    }

    @Test
    fun GIVEN_state_holders_and_compose_scopes_WHEN_checked_THEN_sends_state_holders_to_their_kind(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.root.screen/RootScreenComponent.kt", "interface RootScreenComponent")
            .main("x.root.screen/RootRouter.kt", "interface RootRouter")
            .main("x.offer.api/OfferComponent.kt", "interface OfferComponent")
            .main("x.offer.composable/OfferViewModel.kt", "interface OfferViewModel")
            .main("x.root.component/RootComponent.kt", "interface RootComponent")
            .main(
                "x.filtercard.model/FilterCardScope.kt",
                "import androidx.compose.foundation.layout.ColumnScope\n\ninterface FilterCardScope : ColumnScope"
            )
            .main(
                "x.coroutine.model/WorkScope.kt",
                "import kotlinx.coroutines.CoroutineScope\n\ninterface WorkScope : CoroutineScope"
            )
            .load(UnitKind.CONTEXT, rootPackage = "x")

        val messages = ContractPlacement().check(sources).map(Violation::message).map { message ->
            message.substringBefore(':')
        }

        assertEquals(
            listOf(
                "Move OfferComponent to the component package of its concept",
                "Move OfferViewModel to the viewmodel package of its concept",
                "Move RootRouter to the api package of its concept",
                "Move RootScreenComponent to the component package of its concept",
                "Move WorkScope to the api package of its concept",
            ),
            messages.sorted()
        )
    }

    @Test
    fun GIVEN_read_only_model_interfaces_WHEN_checked_THEN_keeps_data_in_model(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.page.model/Page.kt",
                """
                interface Page<T> {
                    val data: List<T>
                    val total: Long
                    companion object { const val SIZE = 10 }
                }
            """
            )
            .main("x.page.model/Named.kt", "interface Named { val name: String }")
            .main("x.page.model/Labelled.kt", "interface Labelled : Named { val label: String }")
            .load(UnitKind.CONTEXT, rootPackage = "x")

        assertEquals(emptyList(), ContractPlacement().check(sources))
    }

    @Test
    fun GIVEN_behaviour_or_mutability_in_model_interfaces_WHEN_checked_THEN_reports_them(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.page.model/Mutable.kt", "interface Mutable { var name: String }")
            .main("x.page.model/Computed.kt", "interface Computed { val size: Int get() = 1 }")
            .main("x.page.model/Operation.kt", "interface Operation { val name: String; fun load(): String }")
            .main("x.page.model/Child.kt", "interface Child : Operation { val id: Int }")
            .main("x.page.model/Unknown.kt", "interface Unknown : Missing { val id: Int }")
            .main("x.page.model/Empty.kt", "interface Empty")
            .load(UnitKind.CONTEXT, rootPackage = "x")

        assertEquals(
            listOf(
                "Child",
                "Computed",
                "Empty",
                "Mutable",
                "Operation",
                "Unknown"
            ),
            ContractPlacement().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_framework_registration_in_core_WHEN_checked_THEN_keeps_interface_with_mechanism(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.core.route/Routes.kt",
                """
                import io.ktor.server.routing.Routing
                fun interface Routes { fun install(routing: Routing) }
            """
            )
            .main(
                "x.core.command/Commands.kt",
                """
                import org.bukkit.command.CommandMap as CommandsContext
                interface Commands { fun attach(context: CommandsContext): Unit }
            """
            )
            .load(UnitKind.CORE, rootPackage = "x.core")

        assertEquals(emptyList(), ContractPlacement().check(sources))
    }

    @Test
    fun GIVEN_domain_operations_disguised_as_mechanisms_WHEN_checked_THEN_requires_contract_placement(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.core.route/Queries.kt",
                """
                import io.ktor.server.routing.Routing
                interface Queries { fun load(routing: Routing): String }
            """
            )
            .main(
                "x.core.route/Writes.kt",
                """
                import io.ktor.server.routing.Routing
                interface Writes { suspend fun save(routing: Routing) }
            """
            )
            .main(
                "x.core.route/State.kt",
                """
                import io.ktor.server.routing.Routing
                interface State { val routing: Routing; fun install(routing: Routing) }
            """
            )
            .main(
                "x.core.route/Mixed.kt",
                """
                import io.ktor.server.routing.Routing
                interface Mixed { fun install(routing: Routing, user: User) }
            """
            )
            .load(UnitKind.CORE, rootPackage = "x.core")

        assertEquals(
            listOf(
                "Mixed",
                "Queries",
                "State",
                "Writes"
            ),
            ContractPlacement().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_registration_with_unsupported_members_WHEN_checked_THEN_requires_contract_placement(
        @TempDir directory: File
    ) {
        val tree = SourceTree(directory)
        val declarations = listOf(
            "interface Inherited : Parent { fun register(routing: Routing) }",
            "interface DefaultBody { fun register(routing: Routing) {} }",
            "interface NoContext { fun register() }",
            "interface GenericContext { fun register(contexts: List<Routing>) }",
            "interface UnknownContext { fun register(context: Missing) }"
        )
        declarations.forEachIndexed { index, declaration ->
            tree.main(
                "x.core.route/Boundary$index.kt",
                "import io.ktor.server.routing.Routing\n$declaration"
            )
        }
        val sources = tree.load(UnitKind.CORE, rootPackage = "x.core")

        assertEquals(
            listOf(
                "DefaultBody",
                "GenericContext",
                "Inherited",
                "NoContext",
                "UnknownContext"
            ),
            ContractPlacement().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_model_with_stateful_companion_or_non_model_parent_WHEN_checked_THEN_rejects_it(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.page.model/Stateful.kt",
                "interface Stateful { val id: Int; companion object { var count = 0 } }"
            )
            .main(
                "x.page.model/Singleton.kt",
                "interface Singleton { val id: Int; object Cache { var count = 0 } }"
            )
            .main(
                "x.page.model/Initialized.kt",
                "interface Initialized { val id: Int = 1 }"
            )
            .main(
                "x.page.model/Child.kt",
                "import x.other.api.Parent\ninterface Child : Parent { val id: Int }"
            )
            .dependency("x.other.api/Parent.kt", "interface Parent { val name: String }")
            .load(UnitKind.CONTEXT, rootPackage = "x")

        assertEquals(
            listOf(
                "Child",
                "Initialized",
                "Singleton",
                "Stateful"
            ),
            ContractPlacement().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_model_inheritance_and_companions_with_behaviour_WHEN_checked_THEN_does_not_hide_operations(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.page.model/Cyclic.kt", "interface Cyclic : Cyclic { val id: Int }")
            .main(
                "x.page.model/Factory.kt",
                """
                interface Factory {
                    val name: String
                    companion object { fun create() = "value" }
                }
            """
            )
            .main("x.page.model/Callback.kt", "interface Callback { val action: () -> Unit }")
            .main(
                "x.page.model/Child.kt",
                """
                import x.other.model.Parent as Base
                interface Child : Base { val id: Int }
            """
            )
            .dependency("x.other.model/Parent.kt", "interface Parent { val name: String }")
            .load(UnitKind.CONTEXT, rootPackage = "x")

        assertEquals(
            listOf(
                "Callback",
                "Cyclic",
                "Factory"
            ),
            ContractPlacement().check(sources).map(Violation::subject).sorted()
        )
    }
}
