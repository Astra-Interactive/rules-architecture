@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.graph.structure

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import java.io.File
import kotlin.test.assertEquals

internal class CorePlatformDependencyDirectionTest {
    @Test
    fun GIVEN_platform_core_dependencies_WHEN_checked_THEN_only_platform_free_core_is_allowed() {
        val context = UnitContext(
            UnitKind.CORE_PLATFORM,
            ProductKind.PLATFORM_PLUGIN,
            false,
            emptySet(),
            emptyList()
        )
        val dependencies = mapOf(
            ":modules:core:api" to UnitKind.CORE_API,
            ":modules:core:forge" to UnitKind.CORE_PLATFORM,
            ":modules:gui:api" to UnitKind.API,
            ":modules:gui:bukkit" to UnitKind.PLATFORM,
            ":modules:data:dao" to UnitKind.STORAGE_CONTRACT
        )
        val corePlatform = BuildUnit(
            path = ":modules:core:bukkit",
            directory = File("/project"),
            isEntryPoint = false,
            isRoot = false,
            projectDependencies = dependencies.keys,
            context = context,
            dependencyContexts = dependencies.mapValues { (_, kind) -> context.copy(kind = kind) }
        )

        assertEquals(
            listOf(
                ":modules:core:forge",
                ":modules:data:dao",
                ":modules:gui:api",
                ":modules:gui:bukkit"
            ),
            UnitDependencyDirection.findForbidden(corePlatform).sorted()
        )
    }
}
