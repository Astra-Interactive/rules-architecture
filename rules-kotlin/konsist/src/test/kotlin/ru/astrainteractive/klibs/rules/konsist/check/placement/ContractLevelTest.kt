@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class ContractLevelTest {

    @Test
    fun GIVEN_contracts_and_implementations_on_wrong_levels_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("com.example.jail.data", "JailRepository.kt", "interface JailRepository")
            .main("com.example.jail.data.file", "FileJailRepository.kt", "class FileJailRepository : JailRepository")
            .main("com.example.jail.data", "CachedJailRepository.kt", "class CachedJailRepository : JailRepository")
            .main("com.example.jail.model", "BankRepository.kt", "interface BankRepository")
            .main("com.example.jail.data.exposed", "RowRepository.kt", "interface RowRepository")
            .main(
                "com.example.jail.integration",
                "EconomyCharger.kt",
                "interface EconomyCharger\n\nclass VaultEconomyCharger : EconomyCharger"
            )
            .main("com.example.jail.data.database", "RoomJailRepository.kt", "class RoomJailRepository")
            .main("com.example.jail.data.database", "JailDao.kt", "class JailDao")
            .main("com.example.jail.data.remote", "HttpJail.kt", "class HttpJail")
            .main("com.example.jail.presentation", "LegacyRepository.kt", "interface LegacyRepository")
            .load(UnitKind.CONTEXT)

        val subjects = ContractLevel().check(sources).map { violation -> violation.subject }

        assertEquals(
            listOf(
                "BankRepository",
                "CachedJailRepository",
                "RowRepository",
                "VaultEconomyCharger",
                "com.example.jail.data.database",
                "com.example.jail.data.remote",
            ),
            subjects.sorted()
        )
    }
}
