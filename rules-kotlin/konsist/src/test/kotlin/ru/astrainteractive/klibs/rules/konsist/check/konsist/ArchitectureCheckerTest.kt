@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.CheckPolicy
import ru.astrainteractive.klibs.rules.core.unit.model.ImplementationAccess
import ru.astrainteractive.klibs.rules.core.unit.model.RuleId
import ru.astrainteractive.klibs.rules.core.unit.model.UiComposition
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitPolicy
import ru.astrainteractive.klibs.rules.konsist.check.fake.FakeArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.cli.konsist.registeredChecks
import java.io.File
import kotlin.test.assertEquals

internal class ArchitectureCheckerTest {

    @Test
    fun GIVEN_rule_registry_WHEN_compared_with_checker_THEN_every_konsist_rule_is_registered() {
        val expected = RuleId.entries.filter { ruleId -> ruleId.engine == RuleId.Engine.KONSIST }.toSet()

        assertEquals(expected, registeredChecks().keys)
    }

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
        val sources = SourceTree(directory).load(UnitKind.API)

        val violations = ArchitectureChecker(
            mapOf(RuleId.CORE_SHAPE to contextOnly, RuleId.UTIL_SHAPE to everyKind),
            SuppressionPolicy()
        ).check(sources)

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

        val order = ArchitectureChecker(
            mapOf(RuleId.CORE_SHAPE to first, RuleId.UTIL_SHAPE to second),
            SuppressionPolicy()
        ).check(sources)
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

    @Test
    fun GIVEN_suppressed_finding_WHEN_checked_THEN_filters_it_before_returning(@TempDir directory: File) {
        val sourceFile = File(directory, "src/main/kotlin/com/example/Suppressed.kt")
        sourceFile.parentFile.mkdirs()
        sourceFile.writeText(
            """
            @file:Suppress("SuppressedRule")
            package com.example
            class Suppressed
            """.trimIndent()
        )
        val sources = SourceTree(directory).load(UnitKind.CONTEXT)
        val file = sources.mainFiles.single()
        val findings = listOf(
            sources.createViolation(
                "SuppressedRule",
                file,
                "Suppressed",
                "message"
            ),
            sources.createViolation(
                "VisibleRule",
                file,
                "Suppressed",
                "message"
            )
        )
        val check = FakeArchitectureCheck(UnitKind.entries.toSet(), findings)

        val remaining = ArchitectureChecker(mapOf(RuleId.CORE_SHAPE to check), SuppressionPolicy()).check(sources)

        assertEquals(listOf("VisibleRule"), remaining.map(Violation::rule))
    }

    @Test
    fun GIVEN_disabled_rule_WHEN_checked_THEN_skips_only_that_check(@TempDir directory: File) {
        val disabled = FakeArchitectureCheck(
            unitKinds = UnitKind.entries.toSet(),
            violations = listOf(
                Violation(
                    "CoreShape",
                    "a.kt",
                    "A",
                    "message"
                )
            )
        )
        val active = FakeArchitectureCheck(
            unitKinds = UnitKind.entries.toSet(),
            violations = listOf(
                Violation(
                    "UtilShape",
                    "b.kt",
                    "B",
                    "message"
                )
            )
        )
        val sources = SourceTree(directory).load(
            UnitKind.CONTEXT,
            context = UnitContext(
                UnitKind.CONTEXT,
                null,
                false,
                emptySet(),
                emptyList(),
                UnitPolicy(
                    UiComposition.LOCAL,
                    ImplementationAccess.NONE,
                    CheckPolicy(mapOf(RuleId.CORE_SHAPE.ruleName to "Legacy layout"), emptyMap())
                )
            )
        )

        val findings = ArchitectureChecker(
            mapOf(RuleId.CORE_SHAPE to disabled, RuleId.UTIL_SHAPE to active),
            SuppressionPolicy()
        ).check(sources)

        assertEquals(0, disabled.runs)
        assertEquals(1, active.runs)
        assertEquals(listOf("UtilShape"), findings.map(Violation::rule))
    }
}
