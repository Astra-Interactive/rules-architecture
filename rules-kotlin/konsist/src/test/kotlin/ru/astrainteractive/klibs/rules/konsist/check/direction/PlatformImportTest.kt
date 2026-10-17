@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.direction

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class PlatformImportTest {

    @Test
    fun GIVEN_platform_imports_in_model_api_and_usecase_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.jail.jail.model", "Jail.kt", "import org.bukkit.Location\n\nclass Jail")
            .main("x.jail.jail.api", "JailRepository.kt", "import io.ktor.http.*\n\ninterface JailRepository")
            .main(
                "x.jail.jail.usecase",
                "JailPlayerUseCase.kt",
                "import net.minecraft.world.Level\n\nclass JailPlayerUseCase"
            )
            .main(
                "x.jail.jail.command",
                "JailCommand.kt",
                "import org.bukkit.command.Command\n\ninternal class JailCommand"
            )
            .main(
                "x.jail.jail.util",
                "LocationExt.kt",
                """
                import kotlin.time.Duration

                fun format(duration: Duration): String = duration.toString()
                """
            )
            .load(UnitKind.CONTEXT, unitPath = ":jail")

        val subjects = PlatformImport().check(sources).map { violation -> violation.subject }.sorted()

        assertEquals(listOf("io.ktor.http.*", "net.minecraft.world.Level", "org.bukkit.Location"), subjects)
    }

    @Test
    fun GIVEN_api_unit_WHEN_checked_THEN_every_file_is_platform_free(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.gui.api.rating.component",
                "RatingComponent.kt",
                "import org.bukkit.entity.Player\n\nclass RatingComponent"
            )
            .load(UnitKind.API, unitPath = ":gui:api")

        assertEquals(
            listOf("org.bukkit.entity.Player"),
            PlatformImport().check(sources).map { violation -> violation.subject }
        )
    }
}
