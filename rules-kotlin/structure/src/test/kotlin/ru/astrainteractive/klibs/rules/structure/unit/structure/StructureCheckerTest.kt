@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.unit.structure

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.structure.unit.api.StructureCheck
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.model.StructureViolation
import java.io.File
import kotlin.test.assertEquals

internal class StructureCheckerTest {

    @Test
    fun GIVEN_violations_of_several_checks_WHEN_checked_THEN_orders_them_by_path_rule_and_subject() {
        val unit = BuildUnit(
            path = ":modules:jail:impl",
            directory = File("/project"),
            isEntryPoint = false,
            isRoot = false,
            projectDependencies = emptySet()
        )
        val firstViolations = listOf(
            StructureViolation(
                "Zeta",
                "b",
                "B",
                "m"
            ),
            StructureViolation(
                "Alpha",
                "b",
                "C",
                "m"
            )
        )
        val secondViolations = listOf(
            StructureViolation(
                "Alpha",
                "b",
                "A",
                "m"
            ),
            StructureViolation(
                "Zeta",
                "a",
                "Z",
                "m"
            )
        )
        val first = StructureCheck { firstViolations }
        val second = StructureCheck { secondViolations }

        val order = StructureChecker(listOf(first, second)).check(unit)
            .map { violation -> "${violation.path}:${violation.rule}:${violation.subject}" }

        assertEquals(
            listOf(
                "a:Zeta:Z",
                "b:Alpha:A",
                "b:Alpha:C",
                "b:Zeta:B"
            ),
            order
        )
        assertEquals("impl", unit.name)
    }
}
