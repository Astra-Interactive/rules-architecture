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
    fun GIVEN_public_entry_points_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("com.example.jail.command", "JailCommand.kt", "class JailCommand")
            .main("com.example.jail.route", "JailRoutes.kt", "object JailRoutes")
            .main("com.example.jail.event", "JailEvent.kt", "internal class JailEvent\n\nprivate object Helper")
            .main("com.example.jail.service", "UnJailService.kt", "private class UnJailService")
            .main("com.example.jail.model", "Jail.kt", "class Jail")
            .load(UnitKind.CONTEXT)

        val subjects = EntryPointVisibility().check(sources).map { violation -> violation.subject }

        assertEquals(listOf("JailCommand", "JailRoutes"), subjects)
    }
}
