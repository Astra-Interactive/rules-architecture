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
    fun GIVEN_errors_outside_model_and_technology_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("com.example.jail.model", "JailError.kt", "sealed class JailError : Exception()")
            .main("com.example.jail.data.exposed", "ExposedFailure.kt", "class ExposedFailure : RuntimeException()")
            .main("com.example.jail", "JailFailure.kt", "class JailFailure : IllegalStateException()")
            .main(
                "com.example.jail.command",
                "CommandError.kt",
                "class CommandError : JailError()\n\nprivate class LocalError : Throwable()"
            )
            .main("com.example.jail.data", "LoadException.kt", "class LoadException : Exception()")
            .main("com.example.jail.data", "JailRepository.kt", "interface JailRepository : AutoCloseable")
            .main("com.example.jail.presentation", "LegacyError.kt", "class LegacyError : Exception()")
            .load(UnitKind.CONTEXT)

        val subjects = ErrorPlacement().check(sources).map { violation -> violation.subject }

        assertEquals(listOf("CommandError", "JailFailure", "LoadException"), subjects.sorted())
    }
}
