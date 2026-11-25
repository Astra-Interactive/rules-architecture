@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File
import kotlin.test.assertEquals

internal class ImplementationVisibilityTest {

    @Test
    fun GIVEN_public_platform_core_adapter_WHEN_checked_THEN_preserves_existing_core_path_exception(
        @TempDir
        directory: File
    ) {
        val tree = SourceTree(directory).main("x.core.item/ItemStackSerializer.kt", "class ItemStackSerializer")
        val core = tree.load(
            UnitKind.CORE_PLATFORM,
            rootPackage = "x.core",
            unitPath = ":modules:core:bukkit"
        )
        val other = tree.load(
            UnitKind.CORE_PLATFORM,
            rootPackage = "x.core",
            unitPath = ":modules:foundation:bukkit"
        )

        assertEquals(emptyList(), ImplementationVisibility().check(core))
        assertEquals(listOf("ItemStackSerializer"), ImplementationVisibility().check(other).map(Violation::subject))
    }

    @Test
    fun GIVEN_public_implementations_WHEN_checked_THEN_reports_each(
        @TempDir
        directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.jail.api.jail.file/FileJailRepository.kt", "class FileJailRepository")
            .main("x.jail.api.jail.memory/InMemoryJailRepository.kt", "public class InMemoryJailRepository")
            .main("x.jail.api.jail.memory/Cache.kt", "internal class Cache\n\nprivate class Entry")
            .main("x.jail.api.jail.stub/StubJailRepository.kt", "class StubJailRepository")
            .main("x.jail.api.jail.api/JailRepository.kt", "interface JailRepository\n\nclass Holder")
            .load(UnitKind.API, rootPackage = "x.jail.api")

        assertEquals(
            listOf("FileJailRepository"),
            ImplementationVisibility().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_public_generic_class_in_core_WHEN_checked_THEN_reports_nothing(
        @TempDir
        directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.core.cache.cache4k/RefreshingCache.kt", "class RefreshingCache")
            .load(
                UnitKind.CONTEXT,
                rootPackage = "x.core",
                unitPath = ":modules:core:api"
            )

        assertEquals(emptyList(), ImplementationVisibility().check(sources))
    }

    @Test
    fun GIVEN_core_api_unit_on_a_path_without_core_WHEN_checked_THEN_reports_nothing(
        @TempDir
        directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.krate.localstorage/LocalStorageEnumKrate.kt", "class LocalStorageEnumKrate")
            .load(
                UnitKind.CORE_API,
                rootPackage = "x.krate",
                unitPath = ":components:lib-js:krate"
            )

        assertEquals(emptyList(), ImplementationVisibility().check(sources))
    }

    @Test
    fun GIVEN_technology_unit_concept_WHEN_checked_THEN_reports_public_class(
        @TempDir
        directory: File
    ) {
        val sources = SourceTree(directory)
            .main("x.data.exposed.rating/ExposedRatingRepository.kt", "class ExposedRatingRepository")
            .load(UnitKind.TECHNOLOGY, rootPackage = "x.data.exposed")

        assertEquals(
            listOf("Make ExposedRatingRepository internal: the DI module of the unit constructs it."),
            ImplementationVisibility().check(sources).map(Violation::message)
        )
    }

    @Test
    fun GIVEN_metro_annotated_implementation_WHEN_checked_THEN_reports_nothing(
        @TempDir
        directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.jail.api.jail.file/FileJailRepository.kt",
                """
                import dev.zacsweers.metro.ContributesBinding
                import dev.zacsweers.metro.Inject
                import dev.zacsweers.metro.SingleIn

                @Inject
                @SingleIn(AppScope::class)
                @ContributesBinding(AppScope::class)
                class FileJailRepository : JailRepository
                """.trimIndent()
            )
            .load(UnitKind.API, rootPackage = "x.jail.api")

        assertEquals(emptyList(), ImplementationVisibility().check(sources).map(Violation::subject).sorted())
    }

    @Test
    fun GIVEN_constructor_injection_WHEN_checked_THEN_reports_nothing(
        @TempDir
        directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.jail.api.jail.file/FileJailRepository.kt",
                """
                import javax.inject.Inject

                class FileJailRepository @Inject constructor(private val name: String)
                """.trimIndent()
            )
            .main(
                "x.jail.api.jail.file/FileBanRepository.kt",
                """
                import jakarta.inject.Inject

                class FileBanRepository @Inject constructor()
                """.trimIndent()
            )
            .main(
                "x.jail.api.jail.file/FileMuteRepository.kt",
                """
                import me.tatarka.inject.annotations.Inject

                class FileMuteRepository @Inject constructor()
                """.trimIndent()
            )
            .main(
                "x.jail.api.jail.file/FileKickRepository.kt",
                """
                class FileKickRepository(private val id: Int) {
                    @dev.zacsweers.metro.Inject
                    constructor() : this(0)
                }
                """.trimIndent()
            )
            .load(UnitKind.API, rootPackage = "x.jail.api")

        assertEquals(emptyList(), ImplementationVisibility().check(sources).map(Violation::subject).sorted())
    }

    @Test
    fun GIVEN_framework_class_annotations_WHEN_checked_THEN_reports_nothing(
        @TempDir
        directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.jail.api.jail.file/FileJailViewModel.kt",
                """
                import dagger.hilt.android.lifecycle.HiltViewModel

                @HiltViewModel
                class FileJailViewModel
                """.trimIndent()
            )
            .main(
                "x.jail.api.jail.file/FileJailModule.kt",
                """
                import dagger.Module

                @Module
                class FileJailModule
                """.trimIndent()
            )
            .main(
                "x.jail.api.jail.file/FileBanRepository.kt",
                """
                import org.koin.core.annotation.Single

                @Single
                class FileBanRepository
                """.trimIndent()
            )
            .main(
                "x.jail.api.jail.file/FileMuteRepository.kt",
                """
                import org.koin.core.annotation.Factory

                @Factory
                class FileMuteRepository
                """.trimIndent()
            )
            .load(UnitKind.API, rootPackage = "x.jail.api")

        assertEquals(emptyList(), ImplementationVisibility().check(sources).map(Violation::subject).sorted())
    }

    @Test
    fun GIVEN_scoped_framework_annotations_WHEN_checked_THEN_reports_nothing(
        @TempDir
        directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.jail.api.jail.file/FileKickViewModel.kt",
                """
                import org.koin.android.annotation.KoinViewModel

                @KoinViewModel
                class FileKickViewModel
                """.trimIndent()
            )
            .main(
                "x.jail.api.jail.file/FileWarnRepository.kt",
                """
                import jakarta.inject.Singleton

                @Singleton
                class FileWarnRepository
                """.trimIndent()
            )
            .main(
                "x.jail.api.jail.file/FileNoteRepository.kt",
                """
                import dev.zacsweers.metro.ContributesIntoSet

                @ContributesIntoSet(AppScope::class)
                class FileNoteRepository
                """.trimIndent()
            )
            .main(
                "x.jail.api.jail.file/FileTagRepository.kt",
                """
                import dev.zacsweers.metro.ContributesIntoMap as IntoMap

                @IntoMap(AppScope::class)
                class FileTagRepository
                """.trimIndent()
            )
            .load(UnitKind.API, rootPackage = "x.jail.api")

        assertEquals(emptyList(), ImplementationVisibility().check(sources).map(Violation::subject).sorted())
    }

    @Test
    fun GIVEN_same_named_annotation_outside_di_frameworks_WHEN_checked_THEN_reports_the_class(
        @TempDir
        directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.jail.api.jail.file/FileJailRepository.kt",
                """
                import com.example.inject.Inject

                @Inject
                class FileJailRepository
                """.trimIndent()
            )
            .main(
                "x.jail.api.jail.file/FileBanRepository.kt",
                """
                @Singleton
                class FileBanRepository
                """.trimIndent()
            )
            .load(UnitKind.API, rootPackage = "x.jail.api")

        assertEquals(
            listOf("FileBanRepository", "FileJailRepository"),
            ImplementationVisibility().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_di_annotation_only_on_a_member_or_nested_class_WHEN_checked_THEN_reports_the_class(
        @TempDir
        directory: File
    ) {
        val sources = SourceTree(directory)
            .main(
                "x.jail.api.jail.file/FileJailRepository.kt",
                """
                import javax.inject.Inject

                class FileJailRepository {
                    @Inject
                    lateinit var name: String
                }
                """.trimIndent()
            )
            .main(
                "x.jail.api.jail.file/FileBanRepository.kt",
                """
                import dev.zacsweers.metro.Inject

                class FileBanRepository {
                    @Inject
                    class Entry
                }
                """.trimIndent()
            )
            .load(UnitKind.API, rootPackage = "x.jail.api")

        assertEquals(
            listOf("FileBanRepository", "FileJailRepository"),
            ImplementationVisibility().check(sources).map(Violation::subject).sorted()
        )
    }
}
