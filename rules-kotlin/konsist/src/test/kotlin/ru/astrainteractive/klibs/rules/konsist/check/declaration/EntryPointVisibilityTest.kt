@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class EntryPointVisibilityTest {

    @Test
    fun GIVEN_public_classes_in_platform_invoked_kinds_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.command.rating.command", "RatingCommand.kt", "class RatingCommand")
            .main("x.command.error", "CommandExceptionHandler.kt", "object CommandExceptionHandler")
            .main(
                "x.command.reload.command",
                "ReloadCommand.kt",
                "internal class ReloadCommand\n\nprivate object Helper"
            )
            .main("x.command.rating.component", "RatingComponent.kt", "class RatingComponent")
            .main("x.command.di", "CommandsModule.kt", "class CommandsModule")
            .main("x.command.rating.service", "TickService.kt", "private class TickService")
            .load(UnitKind.CONTEXT, unitPath = ":modules:command")

        val subjects = EntryPointVisibility().check(sources).map { violation -> violation.subject }.sorted()

        assertEquals(listOf("CommandExceptionHandler", "RatingCommand"), subjects)
    }
}
