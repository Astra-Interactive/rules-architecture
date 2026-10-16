@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class DiPlacementTest {

    @Test
    fun GIVEN_di_modules_of_a_context_WHEN_checked_THEN_reports_misplaced_and_extra_ones(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("com.example.jail.di", "JailModule.kt", "class JailModule")
            .main("com.example.jail.di", "ExtraModule.kt", "object ExtraModule")
            .main("com.example.jail.di", "JailModuleApi.kt", "interface WiringModule")
            .main("com.example.jail.offer.di", "OfferModule.kt", "class OfferModule")
            .main("com.example.jail.offer.model", "Offer.kt", "class Offer")
            .main("com.example.jail.command.di", "CommandModule.kt", "class CommandModule")
            .main("com.example.jail.model", "StorageModule.kt", "class StorageModule")
            .main("com.example.jail.presentation.di", "LegacyModule.kt", "class LegacyModule")
            .load(UnitKind.CONTEXT)

        val subjects = DiPlacement().check(sources).map { violation -> violation.subject }

        assertEquals(listOf("CommandModule", "JailModule", "StorageModule"), subjects.sorted())
    }

    @Test
    fun GIVEN_di_modules_of_a_library_WHEN_checked_THEN_only_requires_a_di_package(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("com.example.lib.coroutine.di", "CoroutineModule.kt", "class CoroutineModule")
            .main("com.example.lib.coroutine", "ScopeModule.kt", "class ScopeModule")
            .load(UnitKind.LIBRARY)

        val subjects = DiPlacement().check(sources).map { violation -> violation.subject }

        assertEquals(listOf("ScopeModule"), subjects)
    }
}
