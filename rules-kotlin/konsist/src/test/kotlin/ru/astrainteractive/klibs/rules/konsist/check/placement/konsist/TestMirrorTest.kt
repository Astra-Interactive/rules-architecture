@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File
import kotlin.test.assertEquals

internal class TestMirrorTest {

    private fun androidContext(directory: File, compiledTestSourceSet: String): UnitContext {
        val mainDirectory = File(directory, "src/androidMain/kotlin")
        val compiledTestDirectory = File(directory, "src/$compiledTestSourceSet/kotlin")
        val mainSources = mapOf(
            "commonMain" to setOf(File(directory, "src/commonMain/kotlin").absolutePath),
            "androidMain" to setOf(mainDirectory.absolutePath)
        )
        return UnitContext(
            UnitKind.CONTEXT,
            null,
            false,
            emptySet(),
            listOf(
                UnitContext.Compilation(
                    "androidJvm:android:main",
                    mainSources,
                    false,
                    false,
                    emptySet()
                ),
                UnitContext.Compilation(
                    "androidJvm:android:hostTest",
                    mainSources + mapOf(
                        "commonTest" to setOf(File(directory, "src/commonTest/kotlin").absolutePath),
                        compiledTestSourceSet to setOf(compiledTestDirectory.absolutePath)
                    ),
                    true,
                    false,
                    emptySet()
                )
            )
        )
    }

    private fun androidSources(directory: File, testSourceSet: String, compiledTestSourceSet: String): UnitSources {
        val mainDirectory = File(directory, "src/androidMain/kotlin")
        val testDirectory = File(directory, "src/$testSourceSet/kotlin")
        SourceTree(directory)
            .main(
                "androidMain",
                "com.example.wear.di",
                "WearStatusModule.kt",
                "class WearStatusModule"
            )
            .test(
                testSourceSet,
                "com.example.wear.di",
                "WearStatusModuleTest.kt",
                "class WearStatusModuleTest"
            )
        return UnitSources.load(
            kind = UnitKind.CONTEXT,
            rootPackage = null,
            unitPath = null,
            mainDirectories = listOf(mainDirectory),
            testDirectories = listOf(testDirectory),
            mainSourceSets = mapOf("androidMain" to mainDirectory),
            testSourceSets = mapOf(testSourceSet to testDirectory),
            projectDirectory = directory,
            vocabulary = SourceTree.NO_PROJECT_WORDS,
            dependencyDirectories = emptyList(),
            androidNamespace = null,
            context = androidContext(directory, compiledTestSourceSet)
        )
    }

    @Test
    fun GIVEN_tests_named_after_owners_WHEN_checked_THEN_reports_those_in_another_package(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "com.example.jail.model",
                "Jail.kt",
                "class Jail"
            )
            .main(
                "com.example.jail",
                "JailPlayer.kt",
                "class JailPlayer"
            )
            .main(
                "com.example.jail.util",
                "DurationFormat.kt",
                "fun format(seconds: Int): String = \"\""
            )
            .test(
                "com.example.jail.model",
                "JailTest.kt",
                "class JailTest"
            )
            .test(
                "com.example.jail.model",
                "JailPlayerTest.kt",
                "class JailPlayerTest"
            )
            .test(
                "com.example.jail.util",
                "DurationFormatTest.kt",
                "class DurationFormatTest"
            )
            .test(
                "com.example.jail",
                "IntegrationTest.kt",
                "class IntegrationTest"
            )
            .test(
                "com.example.jail.data.fake",
                "FakeJail.kt",
                "class FakeJail"
            )
            .load(UnitKind.CONTEXT)

        val violations = TestMirror().check(sources)

        assertEquals(listOf("JailPlayerTest"), violations.map(Violation::subject))
        assertEquals(
            "Move JailPlayerTest to com.example.jail, the package of JailPlayer.",
            violations.single().message
        )
    }

    @Test
    fun GIVEN_common_main_owner_and_jvm_test_WHEN_checked_THEN_reports_the_source_set(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "commonMain",
                "com.example.jail.model",
                "Jail.kt",
                "class Jail"
            )
            .test(
                "jvmTest",
                "com.example.jail.model",
                "JailTest.kt",
                "class JailTest"
            )
            .loadSourceSets(
                UnitKind.CONTEXT,
                "commonMain",
                "jvmTest"
            )

        assertEquals(
            "Move JailTest to commonTest, the test source set of Jail in com.example.jail.model.",
            TestMirror().check(sources).single().message
        )
    }

    @Test
    fun GIVEN_matching_common_source_sets_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "commonMain",
                "com.example.jail.model",
                "Jail.kt",
                "class Jail"
            )
            .test(
                "commonTest",
                "com.example.jail.model",
                "JailTest.kt",
                "class JailTest"
            )
            .loadSourceSets(
                UnitKind.CONTEXT,
                "commonMain",
                "commonTest"
            )

        assertEquals(emptyList(), TestMirror().check(sources))
    }

    @Test
    fun GIVEN_android_main_owner_and_android_host_test_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        val sources = androidSources(
            directory,
            "androidHostTest",
            "androidHostTest"
        )

        assertEquals(emptyList(), TestMirror().check(sources))
    }

    @Test
    fun GIVEN_android_main_owner_and_legacy_android_unit_test_WHEN_checked_THEN_reports_nothing(
        @TempDir directory: File
    ) {
        val sources = androidSources(
            directory,
            "androidUnitTest",
            "androidUnitTest"
        )

        assertEquals(emptyList(), TestMirror().check(sources))
    }

    @Test
    fun GIVEN_test_source_set_not_in_compilation_WHEN_checked_THEN_does_not_treat_it_as_host_test(
        @TempDir directory: File
    ) {
        val sources = androidSources(
            directory,
            "androidInstrumentedTest",
            "androidHostTest"
        )

        assertEquals(
            "Move WearStatusModuleTest to androidHostTest, the test source set of WearStatusModule in " +
                "com.example.wear.di.",
            TestMirror().check(sources).single().message
        )
    }
}
