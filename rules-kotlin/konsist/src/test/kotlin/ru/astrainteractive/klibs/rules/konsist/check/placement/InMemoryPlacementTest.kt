@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class InMemoryPlacementTest {

    @Test
    fun GIVEN_in_memory_implementations_WHEN_checked_THEN_reports_those_outside_data_memory(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("com.example.jail.data.memory", "InMemoryJailRepository.kt", "class InMemoryJailRepository")
            .main("com.example.jail.data", "InMemoryBankRepository.kt", "class InMemoryBankRepository")
            .main("com.example.jail.model", "InMemoryness.kt", "class InMemoryness")
            .test("com.example.jail.data", "InMemoryShopRepository.kt", "class InMemoryShopRepository")
            .load(UnitKind.CONTEXT)

        val subjects = InMemoryPlacement().check(sources).map { violation -> violation.subject }

        assertEquals(listOf("InMemoryBankRepository", "InMemoryShopRepository"), subjects)
    }
}
