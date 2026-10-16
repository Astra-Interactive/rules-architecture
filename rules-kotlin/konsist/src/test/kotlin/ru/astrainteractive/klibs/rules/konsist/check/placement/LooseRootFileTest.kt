@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class LooseRootFileTest {

    @Test
    fun GIVEN_declarations_at_context_and_module_roots_WHEN_checked_THEN_reports_all_but_use_cases_and_api(
        @TempDir directory: File
    ) {
        val sources = SourceTree(directory)
            .main("com.example.jail", "JailPlayer.kt", "class JailPlayer { operator fun invoke() = Unit }")
            .main("com.example.jail", "JailApi.kt", "interface JailApi")
            .main("com.example.jail", "DefaultJailApi.kt", "class DefaultJailApi : JailApi")
            .main(
                "com.example.jail",
                "Loose.kt",
                """
                data class Jail(val id: Int)
                interface JailRepository
                object Keys
                fun helper() = Unit
                val LIMIT = 10
                typealias JailId = Int
                """
            )
            .main("com.example.jail.offer", "Offer.kt", "class Offer")
            .main("com.example.jail.offer.model", "OfferId.kt", "class OfferId")
            .main("com.example.jail.model", "Cell.kt", "class Cell")
            .load(UnitKind.CONTEXT)

        val subjects = LooseRootFile().check(sources).map { violation -> violation.subject }

        assertEquals(listOf("Jail", "JailRepository", "Keys", "helper", "LIMIT", "JailId", "Offer"), subjects)
    }
}
