@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.detekt.util

import dev.detekt.api.Config
import dev.detekt.test.TestConfig
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.util.encode
import java.io.File
import java.io.FileNotFoundException
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull

// Detekt accepts test settings only as vararg Pair entries; these are external API configuration entries.
@Suppress("AnonymousTuple")
internal class ConfigExtTest {
    @Test
    fun GIVEN_generated_context_WHEN_read_by_detekt_THEN_preserves_metadata_and_project_overrides() {
        val context = UnitContext(
            UnitKind.API,
            ProductKind.APP,
            false,
            setOf("mongo"),
            emptyList()
        )
        val config = TestConfig("unitContext" to context.encode())

        assertEquals(context, config.unitContextOrNull())
        assertEquals(listOf("mongo", "custom"), config.projectTechnologyWords(listOf("mongo", "custom")))
    }

    @Test
    fun GIVEN_standalone_rules_without_gradle_context_WHEN_read_THEN_preserves_local_words() {
        assertNull(Config.empty.unitContextOrNull())
        assertNull(TestConfig("unitContext" to "").unitContextOrNull())
        assertNull(TestConfig("unitContextFile" to "").unitContextOrNull())
        assertEquals(listOf("custom"), Config.empty.projectTechnologyWords(listOf("custom")))
    }

    @Test
    fun GIVEN_corrupt_generated_context_WHEN_read_THEN_fails_instead_of_using_other_facts() {
        assertFailsWith<IllegalArgumentException> {
            TestConfig("unitContext" to "not base64!").unitContextOrNull()
        }
    }

    @Test
    fun GIVEN_context_file_WHEN_read_THEN_preserves_metadata_instead_of_inline_fallback(@TempDir directory: File) {
        val context = UnitContext(
            UnitKind.ENTRY_POINT,
            ProductKind.PLATFORM_PLUGIN,
            false,
            setOf("mongo"),
            emptyList()
        )
        val file = File(directory, "unit-context.txt")
        file.writeText(context.encode())
        val config = TestConfig(
            "unitContextFile" to file.absolutePath,
            "unitContext" to "invalid fallback"
        )

        assertEquals(context, config.unitContextOrNull())
        assertEquals(listOf("mongo", "custom"), config.projectTechnologyWords(listOf("custom")))
    }

    @Test
    fun GIVEN_missing_context_file_WHEN_read_THEN_fails_instead_of_losing_context(@TempDir directory: File) {
        val config = TestConfig("unitContextFile" to File(directory, "missing.txt").absolutePath)

        assertFailsWith<FileNotFoundException> { config.unitContextOrNull() }
    }

    @Test
    fun GIVEN_corrupt_context_file_WHEN_read_THEN_fails_instead_of_losing_context(@TempDir directory: File) {
        val file = File(directory, "unit-context.txt")
        file.writeText("not base64!")
        val config = TestConfig("unitContextFile" to file.absolutePath)

        assertFailsWith<IllegalArgumentException> { config.unitContextOrNull() }
    }
}
