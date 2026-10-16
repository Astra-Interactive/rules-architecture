@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.direction

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class PlatformImportTest {

    @Test
    fun GIVEN_platform_imports_in_use_case_and_contract_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "com.example.jail",
                "JailPlayer.kt",
                """
                import kotlin.time.Duration
                import org.bukkit.Location

                class JailPlayer { operator fun invoke(location: Location, duration: Duration) = Unit }
                """
            )
            .main(
                "com.example.jail.data",
                "JailRepository.kt",
                """
                import io.ktor.http.*

                interface JailRepository
                """
            )
            .main(
                "com.example.jail.data.exposed",
                "ExposedJailRepository.kt",
                """
                import org.jetbrains.exposed.v1.core.Table

                class ExposedJailRepository
                """
            )
            .main("com.example.jail.model", "Jail.kt", "import android.os.Parcelable\n\nclass Jail")
            .load(UnitKind.CONTEXT)

        val subjects = PlatformImport().check(sources).map { violation -> violation.subject }

        assertEquals(listOf("org.bukkit.Location", "io.ktor.http.*"), subjects)
    }
}
