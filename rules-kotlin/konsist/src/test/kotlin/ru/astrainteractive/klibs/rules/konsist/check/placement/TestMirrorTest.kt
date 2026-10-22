@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.placement

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
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
}
