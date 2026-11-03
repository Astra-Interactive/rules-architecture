@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.direction.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File
import kotlin.test.assertEquals

internal class CorePlatformKindImportDirectionTest {
    @Test
    fun GIVEN_platform_core_adapter_imports_WHEN_checked_THEN_enforces_implementation_direction(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.core.item/BukkitItem.kt",
                """
                import x.core.item.api.Item
                import x.core.item.model.ItemKey
                import x.core.item.util.itemName
                import x.core.item.MaterialSerializer
                import x.core.item.di.ItemModule
                import x.core.item.usecase.LoadItemUseCase
                class BukkitItem : Item
                """
            )
            .main("x.core.item.api/Item.kt", "interface Item")
            .main("x.core.item.model/ItemKey.kt", "data class ItemKey(val value: String)")
            .main("x.core.item.util/ItemName.kt", "fun itemName() = \"item\"")
            .main("x.core.item/MaterialSerializer.kt", "object MaterialSerializer")
            .main("x.core.item.di/ItemModule.kt", "class ItemModule")
            .main("x.core.item.usecase/LoadItemUseCase.kt", "class LoadItemUseCase")
            .load(
                UnitKind.CORE_PLATFORM,
                rootPackage = "x.core",
                unitPath = ":modules:core:bukkit"
            )

        assertEquals(
            listOf("x.core.item.di.ItemModule", "x.core.item.usecase.LoadItemUseCase"),
            KindImportDirection().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_platform_core_consumers_WHEN_importing_adapters_THEN_only_di_and_implementations_can_do_it(
        @TempDir directory: File
    ) {
        val tree = SourceTree(directory)
            .main("x.core.item/BukkitItem.kt", "class BukkitItem")
        val rejectedKinds = listOf(
            "api",
            "model",
            "util",
            "usecase"
        )
        (rejectedKinds + "di").forEach { kind ->
            tree.main(
                "x.core.format.$kind/Consumer.kt",
                "import x.core.item.BukkitItem\nclass Consumer"
            )
        }
        val sources = tree.load(
            UnitKind.CORE_PLATFORM,
            rootPackage = "x.core",
            unitPath = ":modules:core:bukkit"
        )

        assertEquals(
            rejectedKinds.map { kind -> "src/main/kotlin/x/core/format/$kind/Consumer.kt" }.sorted(),
            KindImportDirection().check(sources).map(Violation::path).sorted()
        )
    }
}
