@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class TestMirrorTest {

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
}
