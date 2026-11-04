@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.graph.structure

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitRole
import ru.astrainteractive.klibs.rules.core.unit.util.toPolicy
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import java.io.File
import kotlin.test.assertEquals

internal class TechnologyApiDependencyDirectionTest {
    private fun appUnit(
        path: String,
        role: UnitRole,
        dependencies: Map<String, UnitRole>
    ): BuildUnit {
        val context = UnitContext(
            role.kind,
            ProductKind.APP,
            false,
            emptySet(),
            emptyList(),
            role.toPolicy()
        )
        return BuildUnit(
            path = path,
            directory = File("/unrelated/project-directory"),
            isEntryPoint = false,
            isRoot = false,
            projectDependencies = dependencies.keys,
            context = context,
            dependencyContexts = dependencies.mapValues { (_, target) ->
                context.copy(kind = target.kind, policy = target.toPolicy())
            }
        )
    }

    @Test
    fun GIVEN_technology_implementations_WHEN_depending_on_own_api_THEN_accepts_each_technology() {
        listOf("ktor", "exposed").forEach { technology ->
            assertEquals(
                emptyList(),
                UnitDependencyDirection.findForbidden(
                    BuildUnit(
                        path = ":modules:discord-ai:$technology",
                        directory = File("/project"),
                        isEntryPoint = false,
                        isRoot = false,
                        projectDependencies = setOf(":modules:discord-ai:api")
                    )
                )
            )
        }
    }

    @Test
    fun GIVEN_custom_technology_role_WHEN_depending_on_api_roles_THEN_accepts_only_same_immediate_parent() {
        val technology = appUnit(
            ":modules:discord-ai:adapter",
            UnitRole.Technology(),
            mapOf(
                ":modules:discord-ai:exports" to UnitRole.Api(),
                ":modules:discord-ai:nested:exports" to UnitRole.Api(),
                ":modules:discord-ai-extra:exports" to UnitRole.Api(),
                ":modules:other:exports" to UnitRole.Api()
            )
        )

        assertEquals(
            listOf(
                ":modules:discord-ai-extra:exports",
                ":modules:discord-ai:nested:exports",
                ":modules:other:exports"
            ),
            UnitDependencyDirection.findForbidden(technology)
        )
    }

    @Test
    fun GIVEN_technology_role_WHEN_depending_on_other_implementations_THEN_rejects_even_siblings() {
        val technology = appUnit(
            ":modules:discord-ai:adapter",
            UnitRole.Technology(),
            mapOf(
                ":modules:discord-ai:api" to UnitRole.Impl(),
                ":modules:discord-ai:platform" to UnitRole.Platform(),
                ":modules:discord-ai:technology" to UnitRole.Technology(),
                ":modules:discord-ai:fake" to UnitRole.Fake(),
                ":modules:discord-ai:entry" to UnitRole.EntryPoint()
            )
        )

        assertEquals(technology.projectDependencies.sorted(), UnitDependencyDirection.findForbidden(technology))
    }

    @Test
    fun GIVEN_api_role_WHEN_depending_on_sibling_technology_THEN_rejects_reverse_edge() {
        val api = appUnit(
            ":modules:discord-ai:exports",
            UnitRole.Api(),
            mapOf(":modules:discord-ai:adapter" to UnitRole.Technology())
        )

        assertEquals(listOf(":modules:discord-ai:adapter"), UnitDependencyDirection.findForbidden(api))
    }

    @Test
    fun GIVEN_technology_with_foreign_api_WHEN_reported_THEN_explains_own_api_boundary() {
        val technology = appUnit(
            ":modules:discord-ai:adapter",
            UnitRole.Technology(),
            mapOf(":modules:other:exports" to UnitRole.Api())
        )

        assertEquals(
            "Drop :modules:other:exports: a technology unit may depend only on its own api unit with the same " +
                "immediate parent, platform-free core, storage contracts or a context's contract unit.",
            UnitDependency().check(technology).single().message
        )
    }
}
