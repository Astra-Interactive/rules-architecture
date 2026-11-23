@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.util

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.core.unit.model.CheckPolicy
import ru.astrainteractive.klibs.rules.core.unit.model.ImplementationAccess
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.RuleId
import ru.astrainteractive.klibs.rules.core.unit.model.UiComposition
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitPolicy
import kotlin.test.assertEquals

internal class UnitContextExtTest {

    @Test
    fun GIVEN_unit_disabling_a_rule_itself_WHEN_rules_are_disabled_THEN_keeps_its_reason_and_adds_the_rest() {
        val context = UnitContext(
            UnitKind.IMPL,
            ProductKind.APP,
            false,
            emptySet(),
            emptyList()
        ).copy(
            policy = UnitPolicy(
                UiComposition.LOCAL,
                ImplementationAccess.NONE,
                CheckPolicy(
                    mapOf("UnitDependency" to "own reason"),
                    mapOf(":modules:offer:api" to "migration")
                )
            )
        )

        val disabled = context.withDisabledRules(
            listOf(RuleId.UNIT_DEPENDENCY, RuleId.API_UNIT_DEPENDENCY),
            "switch"
        )

        assertEquals(
            context.copy(
                policy = context.policy.copy(
                    checks = CheckPolicy(
                        mapOf("ApiUnitDependency" to "switch", "UnitDependency" to "own reason"),
                        mapOf(":modules:offer:api" to "migration")
                    )
                )
            ),
            disabled
        )
    }
}
