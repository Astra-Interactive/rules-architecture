@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.direction

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class RoleImportDirectionTest {

    private fun SourceTree.withModelAndContracts(): SourceTree = this
        .main(
            "com.example.jail.model",
            "Jail.kt",
            """
            import com.example.jail.data.JailRepository
            import com.example.jail.util.formatName
            import kotlinx.coroutines.*
            import kotlin.time.Duration

            data class Jail(val id: Int, val duration: Duration) {
                data class Cell(val number: Int)
            }
            """
        )
        .main(
            "com.example.jail.util",
            "NameFormat.kt",
            """
            import com.example.jail.model.Jail

            fun formatName(jail: Jail): String = jail.toString()
            """
        )
        .main(
            "com.example.jail.data",
            "JailRepository.kt",
            """
            import com.example.jail.command.*
            import com.example.jail.model.Jail

            interface JailRepository
            """
        )

    private fun SourceTree.withTechnologies(): SourceTree = this
        .main("com.example.jail.data.model", "JailRow.kt", "class JailRow")
        .main("com.example.jail.data.krate", "KrateJailRepository.kt", "class KrateJailRepository")
        .main("com.example.jail.data.exposed.table", "JailTable.kt", "object JailTable")
        .main(
            "com.example.jail.data.exposed",
            "ExposedJailRepository.kt",
            """
            import com.example.jail.JailPlayer
            import com.example.jail.command.JailCommand
            import com.example.jail.data.JailRepository
            import com.example.jail.data.exposed.table.JailTable
            import com.example.jail.data.krate.KrateJailRepository
            import com.example.jail.data.model.JailRow
            import com.example.jail.mapping.toJail
            import com.example.jail.model.Jail.Cell

            class ExposedJailRepository : JailRepository
            """
        )
        .main(
            "com.example.jail.mapping",
            "JailMapping.kt",
            """
            import com.example.jail.data.exposed.ExposedJailRepository
            import com.example.jail.model.Jail

            fun Jail.toJail(): Jail = this
            """
        )

    private fun SourceTree.withRootAndModule(): SourceTree = this
        .main(
            "com.example.jail",
            "JailPlayer.kt",
            """
            import com.example.jail.data.JailRepository
            import com.example.jail.data.exposed.ExposedJailRepository
            import com.example.jail.offer.CreateOffer

            class JailPlayer(private val repository: JailRepository) {
                operator fun invoke() = Unit
            }
            """
        )
        .main("com.example.jail", "JailApi.kt", "interface JailApi")
        .main(
            "com.example.jail",
            "DefaultJailApi.kt",
            """
            import com.example.jail.offer.CreateOffer

            class DefaultJailApi(private val createOffer: CreateOffer) : JailApi
            """
        )
        .main("com.example.jail.offer", "CreateOffer.kt", "class CreateOffer { operator fun invoke() = Unit }")
        .main("com.example.jail.offer.model", "Offer.kt", "class Offer")

    private fun SourceTree.withEntryPoints(): SourceTree = this
        .main(
            "com.example.jail.command",
            "JailCommand.kt",
            """
            import com.example.jail.JailPlayer
            import com.example.jail.data.JailRepository
            import com.example.jail.di.JailModule
            import com.example.jail.event.JailEvent

            internal class JailCommand
            """
        )
        .main(
            "com.example.jail.command.admin",
            "AdminCommand.kt",
            """
            import com.example.jail.command.JailCommand

            internal class AdminCommand
            """
        )
        .main("com.example.jail.event", "JailEvent.kt", "internal class JailEvent")
        .main(
            "com.example.jail.composable",
            "JailScreen.kt",
            """
            import com.example.jail.viewmodel.JailViewModel

            fun JailScreen(viewModel: JailViewModel) = Unit
            """
        )
        .main("com.example.jail.viewmodel", "JailViewModel.kt", "internal class JailViewModel")

    private fun SourceTree.withWiringAndLegacy(): SourceTree = this
        .main(
            "com.example.jail.integration.vault",
            "VaultJailProvider.kt",
            """
            import com.example.jail.JailPlayer
            import com.example.jail.data.exposed.ExposedJailRepository

            class VaultJailProvider
            """
        )
        .main(
            "com.example.jail.di",
            "JailModule.kt",
            """
            import com.example.jail.JailPlayer
            import com.example.jail.command.JailCommand
            import com.example.jail.data.exposed.ExposedJailRepository

            class JailModule
            """
        )
        .main(
            "com.example.jail.presentation",
            "Legacy.kt",
            """
            import com.example.jail.data.exposed.ExposedJailRepository

            class Legacy
            """
        )

    private fun jailContext(directory: File): SourceTree = SourceTree(directory)
        .withModelAndContracts()
        .withTechnologies()
        .withRootAndModule()
        .withEntryPoints()
        .withWiringAndLegacy()

    @Test
    fun GIVEN_context_with_imports_against_the_direction_table_WHEN_checked_THEN_reports_each_one(
        @TempDir directory: File
    ) {
        val sources = jailContext(directory).load(UnitKind.CONTEXT)

        val reported = RoleImportDirection().check(sources)
            .map { violation -> "${violation.path.substringAfter("com/example/jail/")} -> ${violation.subject}" }
            .sorted()

        assertEquals(
            listOf(
                "JailPlayer.kt -> com.example.jail.data.exposed.ExposedJailRepository",
                "JailPlayer.kt -> com.example.jail.offer.CreateOffer",
                "command/JailCommand.kt -> com.example.jail.di.JailModule",
                "command/JailCommand.kt -> com.example.jail.event.JailEvent",
                "data/JailRepository.kt -> com.example.jail.command.*",
                "data/exposed/ExposedJailRepository.kt -> com.example.jail.JailPlayer",
                "data/exposed/ExposedJailRepository.kt -> com.example.jail.command.JailCommand",
                "data/exposed/ExposedJailRepository.kt -> com.example.jail.data.krate.KrateJailRepository",
                "integration/vault/VaultJailProvider.kt -> com.example.jail.data.exposed.ExposedJailRepository",
                "mapping/JailMapping.kt -> com.example.jail.data.exposed.ExposedJailRepository",
                "model/Jail.kt -> com.example.jail.data.JailRepository",
            ),
            reported
        )
    }

    @Test
    fun GIVEN_violation_WHEN_reported_THEN_message_names_both_places(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("com.example.jail.model", "Jail.kt", "import com.example.jail.data.JailRepository\n\nclass Jail")
            .main("com.example.jail.data", "JailRepository.kt", "interface JailRepository")
            .load(UnitKind.CONTEXT)

        val violation = RoleImportDirection().check(sources).single()

        assertEquals("RoleImportDirection", violation.rule)
        assertEquals(
            "'model' must not import com.example.jail.data.JailRepository from the data contract.",
            violation.message
        )
    }
}
