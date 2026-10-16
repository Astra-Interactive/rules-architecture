@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class DefaultApiImplementsApiTest {

    @Test
    fun GIVEN_facades_WHEN_checked_THEN_reports_missing_interface_and_wrong_place(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("com.example.jail", "JailApi.kt", "interface JailApi")
            .main("com.example.jail", "DefaultJailApi.kt", "class DefaultJailApi : JailApi")
            .main("com.example.jail", "DefaultShopApi.kt", "class DefaultShopApi")
            .main("com.example.jail.model", "DefaultBankApi.kt", "class DefaultBankApi : BankApi")
            .main("com.example.jail.presentation", "DefaultLegacyApi.kt", "class DefaultLegacyApi : LegacyApi")
            .load(UnitKind.CONTEXT)

        val messages = DefaultApiImplementsApi().check(sources).map { violation -> violation.message }

        assertEquals(
            listOf(
                "DefaultShopApi must implement ShopApi, the interface it is the one implementation of.",
                "Move DefaultBankApi to the context root, next to the use cases it delegates to.",
            ),
            messages
        )
    }
}
