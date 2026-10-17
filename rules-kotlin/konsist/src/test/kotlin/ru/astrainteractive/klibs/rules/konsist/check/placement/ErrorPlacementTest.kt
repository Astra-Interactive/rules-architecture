@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class ErrorPlacementTest {

    @Test
    fun GIVEN_errors_outside_model_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.command.model", "UsageCommandException.kt", "class UsageCommandException : Exception()")
            .main("x.command.rating.model", "RatingError.kt", "sealed class RatingError : Exception()")
            .main("x.command.error", "HandlerFailure.kt", "class HandlerFailure : IllegalStateException()")
            .main(
                "x.command.rating.command",
                "RatingCommand.kt",
                "internal class RatingCommand\n\nprivate class LocalError : Throwable()"
            )
            .main("x.command.rating.api", "RatingApi.kt", "interface RatingApi : AutoCloseable")
            .load(UnitKind.CONTEXT, unitPath = ":modules:command")

        val subjects = ErrorPlacement().check(sources).map { violation -> violation.subject }.sorted()

        assertEquals(listOf("HandlerFailure", "LocalError"), subjects)
    }
}
