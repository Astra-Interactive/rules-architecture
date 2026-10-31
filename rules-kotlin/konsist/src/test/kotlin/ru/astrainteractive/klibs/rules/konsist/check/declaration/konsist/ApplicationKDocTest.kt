@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.declaration.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureChecker
import ru.astrainteractive.klibs.rules.konsist.check.konsist.SuppressionPolicy
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class ApplicationKDocTest {
    private fun writeSources(directory: File): SourceTree {
        return SourceTree(directory)
            .main(
                "x.jail.api.jail.api/JailRepository.kt",
                """
                /**
                 * Exposes the jails so the use cases obtain them through the module.
                 */
                interface JailRepository {
                    /**
                     * @return failure with [JailError.NotFound] when no jail has this id
                     */
                    suspend fun deleteJail(id: JailId): Result<Unit>

                    /**
                     * Loads a jail.
                     */
                    suspend fun findJail(id: JailId): Jail?
                }
                """
            )
            .main(
                "x.jail.api.jail.file/FileJailRepository.kt",
                """
                /**
                 * Bukkit fires the event twice on a reload (a server quirk), so the second call is ignored.
                 */
                internal class FileJailRepository

                /**
                 * See https://youtrack.jetbrains.com/issue/KT-12345 for why the type is spelled out.
                 */
                internal class TypedJail

                internal class Undocumented
                """
            )
    }

    @Test
    fun GIVEN_kdoc_in_application_code_WHEN_checked_THEN_reports_what_restates_the_code(@TempDir directory: File) {
        val sources = writeSources(directory).load(UnitKind.API, rootPackage = "x.jail.api")

        assertEquals(
            listOf("JailRepository", "findJail"),
            ApplicationKDoc().check(sources).map(Violation::subject).sorted()
        )
    }

    @Test
    fun GIVEN_published_library_WHEN_checked_by_the_checker_THEN_its_kdoc_is_not_judged(@TempDir directory: File) {
        val sources = writeSources(directory).load(UnitKind.LIBRARY, rootPackage = "x.jail.api")
        val checker = ArchitectureChecker(listOf(ApplicationKDoc()), SuppressionPolicy())

        assertEquals(emptyList(), checker.check(sources))
    }

    @Test
    fun GIVEN_class_kdoc_and_constructor_properties_WHEN_checked_THEN_reports_each_kdoc_once(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.core.settings.model/Translation.kt",
                """
                class Translation(
                    val commandError: CommandError
                ) {
                    /** Failures any command can report. */
                    data class CommandError(
                        val unknownError: String = "Unknown error",
                        val wrongUsage: String = "Wrong usage",
                        /** Shown when the sender lacks the permission. */
                        val noPermission: String = "No permission"
                    )
                }
                """
            )
            .load(UnitKind.CONTEXT, rootPackage = "x.core")

        assertEquals(
            listOf("CommandError", "noPermission"),
            ApplicationKDoc().check(sources).map(Violation::subject).sorted()
        )
    }
}
