@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.direction.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File
import kotlin.test.assertEquals
import kotlin.test.assertTrue

internal class PlatformUnitBoundaryTest {
    @Test
    fun GIVEN_platform_and_technology_roles_WHEN_scheduling_checks_THEN_keeps_platform_boundaries() {
        val rule = PlatformImport()
        assertTrue(UnitKind.PLATFORM in rule.unitKinds)
        assertTrue(UnitKind.TECHNOLOGY in rule.unitKinds)
        assertTrue(UnitKind.CORE_PLATFORM in rule.unitKinds)
    }

    @Test
    fun GIVEN_platform_roles_WHEN_contracts_models_and_usecases_import_frameworks_THEN_reports_them(
        @TempDir directory: File
    ) {
        val tree = SourceTree(directory)
            .main(
                "x.generator.price.api/PriceSource.kt",
                "import org.bukkit.Material\ninterface PriceSource { fun price(material: Material): Double }"
            )
            .main(
                "x.generator.recipe.model/Recipe.kt",
                "data class Recipe(val item: org.bukkit.Material)"
            )
            .main(
                "x.generator.price.usecase/GeneratePricesUseCase.kt",
                "import org.jetbrains.exposed.sql.Table\nclass GeneratePricesUseCase"
            )
            .main(
                "x.generator.price.poi/Workbook.kt",
                "import org.bukkit.Material\nclass Workbook"
            )
            .main(
                "x.generator.recipe/BukkitRecipeProvider.kt",
                "import org.bukkit.Bukkit\nclass BukkitRecipeProvider"
            )
            .main(
                "x.generator.recipe.bukkit/RecipeIngredient.kt",
                "import org.bukkit.Material\ndata class RecipeIngredient(val item: Material)"
            )
            .main(
                "x.generator.craft.command/CraftCommand.kt",
                "import org.bukkit.Material\nclass CraftCommand"
            )
            .main(
                "x.generator.di/GeneratorModule.kt",
                "import org.bukkit.Bukkit\nclass GeneratorModule"
            )
        listOf(
            UnitKind.PLATFORM,
            UnitKind.TECHNOLOGY,
            UnitKind.CORE_PLATFORM
        ).forEach { role ->
            val sources = tree.load(role, rootPackage = "x.generator")
            assertEquals(
                listOf(
                    "org.bukkit.Material",
                    "org.bukkit.Material",
                    "org.jetbrains.exposed.sql.Table"
                ),
                PlatformImport().check(sources).map(Violation::subject).sorted(),
                role.name
            )
        }
    }

    @Test
    fun GIVEN_platform_model_with_framework_exception_WHEN_checked_THEN_allows_only_exception_base(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.command.error.model/CommandFailure.kt",
                "import org.bukkit.command.CommandException\nclass CommandFailure : CommandException()"
            )
            .main(
                "x.command.error.model/FailureLocation.kt",
                "import org.bukkit.Location\ndata class FailureLocation(val value: Location)"
            )
            .load(UnitKind.PLATFORM, rootPackage = "x.command")

        assertEquals(listOf("org.bukkit.Location"), PlatformImport().check(sources).map(Violation::subject))
    }
}
