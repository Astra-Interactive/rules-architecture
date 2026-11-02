@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.direction.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File
import kotlin.test.assertEquals

internal class KindImportDirectionContractTest {
    @Test
    fun GIVEN_state_holder_and_usecase_contracts_WHEN_used_by_stubs_THEN_allows_interfaces_only(
        @TempDir directory: File
    ) {
        listOf(
            "component",
            "viewmodel",
            "usecase"
        ).forEach { kind ->
            val sources = SourceTree(directory.resolve(kind))
                .main(
                    "x.feature.$kind/Contract.kt",
                    """
                    interface Contract
                    class DefaultContract : Contract
                    class MisleadingComponent
                    """
                )
                .main(
                    "x.feature.stub/StubContract.kt",
                    """
                    import x.feature.$kind.Contract as Boundary
                    import x.feature.$kind.DefaultContract
                    import x.feature.$kind.MisleadingComponent

                    class StubContract : Boundary {
                        val delegate = DefaultContract()
                        val unrelated = MisleadingComponent()
                    }
                    """
                )
                .load(UnitKind.IMPL, rootPackage = "x.feature")

            assertEquals(
                listOf("x.feature.$kind.DefaultContract", "x.feature.$kind.MisleadingComponent"),
                KindImportDirection().check(sources).map(Violation::subject),
                kind
            )
        }
    }

    @Test
    fun GIVEN_nested_component_contract_WHEN_stub_uses_qualified_reference_THEN_keeps_nested_classes_forbidden(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.splash.component/SplashComponent.kt",
                """
                interface SplashComponent {
                    fun interface Factory {
                        fun create(): SplashComponent
                    }
                    class DefaultFactory
                }
                """
            )
            .main(
                "x.splash.stub/StubSplashFactory.kt",
                """
                import x.splash.component.SplashComponent.Factory
                import x.splash.component.SplashComponent.DefaultFactory

                class StubSplashFactory : Factory {
                    override fun create(): x.splash.component.SplashComponent = TODO()
                    val delegate = DefaultFactory()
                }
                """
            )
            .load(UnitKind.API, rootPackage = "x.splash")

        assertEquals(
            listOf("x.splash.component.SplashComponent.DefaultFactory"),
            KindImportDirection().check(sources).map(Violation::subject)
        )
    }

    @Test
    fun GIVEN_component_contract_WHEN_referenced_by_each_kind_THEN_applies_contract_direction(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.feature.component/Component.kt",
                "interface Component"
            )
            .main(
                "x.feature.stub/StubComponent.kt",
                "import x.feature.component.Component\nclass StubComponent : Component"
            )
            .main(
                "x.feature.memory/Memory.kt",
                "import x.feature.component.Component\nclass Memory(val value: Component)"
            )
            .main(
                "x.feature.util/ValueExt.kt",
                "import x.feature.component.Component\nfun Component.value(): Int = 1"
            )
            .main(
                "x.feature.model/Value.kt",
                "import x.feature.component.Component\nclass Value(val value: Component)"
            )
            .main(
                "x.feature.theme/Theme.kt",
                "import x.feature.component.Component\nclass Theme(val value: Component)"
            )
            .main(
                "x.feature.api/Other.kt",
                "import x.feature.component.Component\ninterface Other : Component"
            )
            .load(UnitKind.IMPL, rootPackage = "x.feature")

        assertEquals(
            listOf(
                "api/Other.kt",
                "model/Value.kt",
                "theme/Theme.kt"
            ),
            KindImportDirection().check(sources).map { violation ->
                violation.path.substringAfter("x/feature/")
            }.sorted()
        )
    }

    @Test
    fun GIVEN_interfaces_outside_contract_homes_WHEN_imported_by_stub_THEN_does_not_exempt_them(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.feature.di/Module.kt",
                "interface Module"
            )
            .main(
                "x.feature.command/Command.kt",
                "interface Command"
            )
            .main(
                "x.feature.stub/StubContract.kt",
                """
                import x.feature.di.Module
                import x.feature.command.Command

                class StubContract(val module: Module, val command: Command)
                """
            )
            .load(UnitKind.IMPL, rootPackage = "x.feature")

        assertEquals(
            listOf("x.feature.di.Module", "x.feature.command.Command"),
            KindImportDirection().check(sources).map(Violation::subject)
        )
    }

    @Test
    fun GIVEN_usecase_interface_WHEN_called_by_another_usecase_THEN_keeps_the_direction_forbidden(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.feature.usecase/LoadUseCase.kt",
                "interface LoadUseCase"
            )
            .main(
                "x.feature.other.usecase/SaveUseCase.kt",
                """
                import x.feature.usecase.LoadUseCase
                class SaveUseCase(val load: LoadUseCase)
                """
            )
            .load(UnitKind.IMPL, rootPackage = "x.feature")

        assertEquals(
            listOf("x.feature.usecase.LoadUseCase"),
            KindImportDirection().check(sources).map(Violation::subject)
        )
    }
}
