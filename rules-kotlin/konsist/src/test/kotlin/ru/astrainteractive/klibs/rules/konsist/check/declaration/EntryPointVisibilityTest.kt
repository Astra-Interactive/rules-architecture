@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureChecker
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class EntryPointVisibilityTest {

    @Test
    fun GIVEN_public_classes_in_platform_invoked_kinds_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.command.rating.command",
                "RatingCommand.kt",
                "class RatingCommand"
            )
            .main(
                "x.command.error",
                "CommandExceptionHandler.kt",
                "object CommandExceptionHandler"
            )
            .main(
                "x.command.reload.command",
                "ReloadCommand.kt",
                "internal class ReloadCommand\n\nprivate object Helper"
            )
            .main(
                "x.command.rating.component",
                "RatingComponent.kt",
                "class RatingComponent"
            )
            .main(
                "x.command.rating.gui",
                "RatingMenu.kt",
                "class RatingMenu"
            )
            .main(
                "x.command.rating.menu",
                "RatingScreen.kt",
                "class RatingScreen"
            )
            .main(
                "x.command.di",
                "CommandsModule.kt",
                "class CommandsModule"
            )
            .main(
                "x.command.rating.service",
                "TickService.kt",
                "private class TickService"
            )
            .main(
                "x.command.papi.expansion",
                "RatingExpansion.kt",
                "class RatingExpansion"
            )
            .load(UnitKind.CONTEXT, unitPath = ":modules:command")

        val subjects = EntryPointVisibility().check(sources).map(Violation::subject).sorted()

        assertEquals(
            listOf(
                "CommandExceptionHandler",
                "RatingCommand",
                "RatingExpansion",
                "RatingMenu",
                "RatingScreen"
            ),
            subjects
        )
    }

    @Test
    fun GIVEN_published_library_WHEN_checked_by_the_checker_THEN_its_public_command_api_is_not_judged(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.klibs.command.command",
                "Command.kt",
                "class Command"
            )
            .main(
                "x.klibs.command.event",
                "EventListener.kt",
                "class EventListener"
            )
            .load(UnitKind.LIBRARY, unitPath = ":command")

        assertEquals(emptyList(), ArchitectureChecker(listOf(EntryPointVisibility())).check(sources))
    }
}
