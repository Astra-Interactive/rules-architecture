@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.konsist.check.fake.FakeArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import java.io.File
import kotlin.test.assertEquals

internal class ArchitectureCheckerTest {

    @Test
    fun GIVEN_checks_for_other_kinds_WHEN_checked_THEN_runs_only_those_of_the_unit_kind(@TempDir directory: File) {
        val contextOnly = FakeArchitectureCheck(
            unitKinds = setOf(UnitKind.CONTEXT),
            violations = listOf(
                Violation(
                    "ContextRule",
                    "a.kt",
                    "A",
                    "context"
                )
            )
        )
        val everyKind = FakeArchitectureCheck(unitKinds = UnitKind.entries.toSet(), violations = emptyList())
        val sources = SourceTree(directory).load(UnitKind.LIBRARY)

        val violations = ArchitectureChecker(listOf(contextOnly, everyKind)).check(sources)

        assertEquals(emptyList(), violations)
        assertEquals(0, contextOnly.runs)
        assertEquals(1, everyKind.runs)
    }

    @Test
    fun GIVEN_violations_of_several_checks_WHEN_checked_THEN_orders_them_by_path_rule_and_subject(
        @TempDir directory: File
    ) {
        val first = FakeArchitectureCheck(
            unitKinds = setOf(UnitKind.CONTEXT),
            violations = listOf(
                Violation(
                    "Zeta",
                    "b.kt",
                    "B",
                    "message"
                ),
                Violation(
                    "Alpha",
                    "b.kt",
                    "C",
                    "message"
                ),
            )
        )
        val second = FakeArchitectureCheck(
            unitKinds = setOf(UnitKind.CONTEXT),
            violations = listOf(
                Violation(
                    "Alpha",
                    "b.kt",
                    "A",
                    "message"
                ),
                Violation(
                    "Zeta",
                    "a.kt",
                    "Z",
                    "message"
                ),
            )
        )
        val sources = SourceTree(directory).load(UnitKind.CONTEXT)

        val order = ArchitectureChecker(listOf(first, second)).check(sources)
            .map { violation -> "${violation.path}:${violation.rule}:${violation.subject}" }

        assertEquals(
            listOf(
                "a.kt:Zeta:Z",
                "b.kt:Alpha:A",
                "b.kt:Alpha:C",
                "b.kt:Zeta:B"
            ),
            order
        )
    }
}
