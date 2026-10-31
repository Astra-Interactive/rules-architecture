@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class DiModuleVisibilityTest {

    private fun SourceTree.withRatingModules(): SourceTree {
        return main(
            "x.data.exposed.di/ExposedModule.kt",
            "class ExposedModule {\n    val ratingStorageModule = RatingStorageModule(this)\n}"
        )
            .main("x.data.exposed.rating.di/RatingStorageModule.kt", "class RatingStorageModule(parent: Any)")
            .main("x.data.exposed.cache.di/CacheModule.kt", "internal class CacheModule")
            .main("x.data.exposed.cache/CacheUser.kt", "internal class CacheUser {\n    val module = CacheModule()\n}")
    }

    @Test
    fun GIVEN_public_child_module_built_in_its_own_unit_WHEN_checked_THEN_reports_it(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .withRatingModules()
            .load(UnitKind.TECHNOLOGY, unitPath = ":data:exposed")

        val violation = DiModuleVisibility().check(sources).single()

        assertEquals("RatingStorageModule", violation.subject)
        assertEquals(
            "Make RatingStorageModule internal: a DI module of this unit constructs it, and only a DI module " +
                "another unit constructs is public.",
            violation.message
        )
    }

    @Test
    fun GIVEN_check_WHEN_listing_its_unit_kinds_THEN_leaves_out_published_libraries() {
        assertEquals(false, UnitKind.LIBRARY in DiModuleVisibility().unitKinds)
    }
}
