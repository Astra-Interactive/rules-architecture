@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class ImplementationVisibilityTest {

    @Test
    fun GIVEN_public_implementations_WHEN_checked_THEN_reports_each(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.jail.api.jail.file/FileJailRepository.kt", "class FileJailRepository")
            .main("x.jail.api.jail.memory/InMemoryJailRepository.kt", "public class InMemoryJailRepository")
            .main("x.jail.api.jail.memory/Cache.kt", "internal class Cache\n\nprivate class Entry")
            .main("x.jail.api.jail.stub/StubJailRepository.kt", "class StubJailRepository")
            .main("x.jail.api.jail.api/JailRepository.kt", "interface JailRepository\n\nclass Holder")
            .load(UnitKind.API, rootPackage = "x.jail.api")

        assertEquals(
            listOf("FileJailRepository", "InMemoryJailRepository"),
            ImplementationVisibility().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_technology_unit_concept_WHEN_checked_THEN_reports_public_class(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.data.exposed.rating/ExposedRatingRepository.kt", "class ExposedRatingRepository")
            .load(UnitKind.TECHNOLOGY, rootPackage = "x.data.exposed")

        assertEquals(
            listOf("Make ExposedRatingRepository internal: the DI module of the unit constructs it."),
            ImplementationVisibility().check(sources).map(Violation::message)
        )
    }
}
