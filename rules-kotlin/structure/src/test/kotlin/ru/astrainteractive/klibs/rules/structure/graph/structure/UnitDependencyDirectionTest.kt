@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.structure.graph.structure

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.core.unit.model.CheckPolicy
import ru.astrainteractive.klibs.rules.core.unit.model.ImplementationAccess
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UiComposition
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitRole
import ru.astrainteractive.klibs.rules.core.unit.util.toPolicy
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import java.io.File
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

internal class UnitDependencyDirectionTest {
    private fun forbidden(
        path: String,
        dependencies: Set<String>,
        isEntryPoint: Boolean = false,
        isLibrary: Boolean = false
    ): List<String> {
        val unit = BuildUnit(
            path = path,
            directory = File("/project"),
            isEntryPoint = isEntryPoint,
            isRoot = false,
            projectDependencies = dependencies,
            isLibrary = isLibrary
        )
        return UnitDependencyDirection.findForbidden(unit)
    }

    private fun explicitUnit(
        path: String,
        kind: UnitKind,
        dependencies: Map<String, UnitKind>,
        isLibrary: Boolean = false
    ): BuildUnit {
        val context = UnitContext(
            kind,
            ProductKind.PLATFORM_PLUGIN,
            isLibrary,
            emptySet(),
            emptyList()
        )
        return BuildUnit(
            path = path,
            directory = File("/project"),
            isEntryPoint = false,
            isRoot = false,
            projectDependencies = dependencies.keys,
            isLibrary = isLibrary,
            context = context,
            dependencyContexts = dependencies.mapValues { (_, dependencyKind) ->
                context.copy(kind = dependencyKind)
            }
        )
    }

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
    fun GIVEN_local_ui_with_declared_sibling_impls_WHEN_checked_THEN_accepts_each_role_edge() {
        val ui = appUnit(
            ":features:rating:ui",
            UnitRole.ImplUi(),
            mapOf(
                ":features:rating:impl" to UnitRole.Impl(),
                ":features:rating:resources" to UnitRole.Impl()
            )
        )

        assertEquals(emptyList(), UnitDependencyDirection.findForbidden(ui))
    }

    @Test
    fun GIVEN_local_ui_with_foreign_or_prefix_matched_impl_WHEN_checked_THEN_rejects_both() {
        val ui = appUnit(
            ":features:rating:ui",
            UnitRole.ImplUi(),
            mapOf(
                ":features:ratingExtra:impl" to UnitRole.Impl(),
                ":features:towns:impl" to UnitRole.Impl()
            )
        )

        assertEquals(ui.projectDependencies.sorted(), UnitDependencyDirection.findForbidden(ui))
    }

    @Test
    fun GIVEN_sibling_path_with_wrong_target_role_WHEN_checked_THEN_rejects_the_edge() {
        val ui = appUnit(
            ":features:rating:ui",
            UnitRole.ImplUi(),
            mapOf(":features:rating:impl" to UnitRole.Technology())
        )

        assertEquals(listOf(":features:rating:impl"), UnitDependencyDirection.findForbidden(ui))
    }

    @Test
    fun GIVEN_unused_sibling_impl_metadata_WHEN_checked_THEN_does_not_create_an_edge() {
        val ui = appUnit(
            ":features:rating:ui",
            UnitRole.ImplUi(),
            emptyMap()
        )
        val sibling = UnitContext(
            UnitKind.IMPL,
            ProductKind.APP,
            false,
            emptySet(),
            emptyList()
        )
        val withUnusedSibling = BuildUnit(
            ui.path,
            ui.directory,
            false,
            false,
            emptySet(),
            context = ui.context,
            dependencyContexts = mapOf(":features:rating:impl" to sibling)
        )

        assertEquals(emptyList(), UnitDependencyDirection.findForbidden(withUnusedSibling))
    }

    @Test
    fun GIVEN_local_ui_without_sibling_access_WHEN_checked_THEN_rejects_sibling_impl() {
        val ui = appUnit(
            ":features:rating:ui",
            UnitRole.ImplUi(implementationAccess = ImplementationAccess.NONE),
            mapOf(":features:rating:impl" to UnitRole.Impl())
        )

        assertEquals(listOf(":features:rating:impl"), UnitDependencyDirection.findForbidden(ui))
    }

    @Test
    fun GIVEN_host_and_local_ui_WHEN_depending_on_other_feature_ui_THEN_only_host_accepts_it() {
        val dependencies = mapOf(":features:rating:ui" to UnitRole.ImplUi())
        val host = appUnit(
            ":root:ui",
            UnitRole.ImplUi(composition = UiComposition.HOST),
            dependencies
        )
        val local = appUnit(
            ":root:ui",
            UnitRole.ImplUi(),
            dependencies
        )

        assertEquals(emptyList(), UnitDependencyDirection.findForbidden(host))
        assertEquals(listOf(":features:rating:ui"), UnitDependencyDirection.findForbidden(local))
    }

    @Test
    fun GIVEN_host_ui_WHEN_depending_on_foreign_impl_THEN_rejects_it() {
        val host = appUnit(
            ":root:ui",
            UnitRole.ImplUi(composition = UiComposition.HOST),
            mapOf(":features:rating:impl" to UnitRole.Impl())
        )

        assertEquals(listOf(":features:rating:impl"), UnitDependencyDirection.findForbidden(host))
        assertEquals(
            true,
            UnitDependency().check(host).single().message.contains("same immediate parent")
        )
    }

    @Test
    fun GIVEN_local_ui_with_foreign_ui_WHEN_reported_THEN_names_host_composition_requirement() {
        val local = appUnit(
            ":root:ui",
            UnitRole.ImplUi(),
            mapOf(":features:rating:ui" to UnitRole.ImplUi())
        )

        assertEquals(
            true,
            UnitDependency().check(local).single().message.contains("HOST composition mode")
        )
    }

    @Test
    fun GIVEN_app_core_and_feature_roles_WHEN_checked_THEN_core_is_inbound_only() {
        val core = appUnit(
            ":services:core:common",
            UnitRole.Core(),
            mapOf(":features:rating:api" to UnitRole.Api())
        )
        val api = appUnit(
            ":features:rating:api",
            UnitRole.Api(),
            mapOf(":services:core:common" to UnitRole.Core())
        )

        assertEquals(listOf(":features:rating:api"), UnitDependencyDirection.findForbidden(core))
        assertEquals(emptyList(), UnitDependencyDirection.findForbidden(api))
    }

    @Test
    fun GIVEN_exact_dependency_exception_WHEN_checked_THEN_only_that_declared_edge_is_allowed() {
        val ui = appUnit(
            ":features:rating:ui",
            UnitRole.ImplUi(),
            mapOf(
                ":features:towns:impl" to UnitRole.Impl(),
                ":features:status:impl" to UnitRole.Impl()
            )
        )
        val policy = ui.context!!.policy.copy(
            checks = CheckPolicy(emptyMap(), mapOf(":features:towns:impl" to "temporary direct resource use"))
        )
        val excepted = BuildUnit(
            path = ui.path,
            directory = ui.directory,
            isEntryPoint = false,
            isRoot = false,
            projectDependencies = ui.projectDependencies,
            context = ui.context.copy(policy = policy),
            dependencyContexts = ui.dependencyContexts
        )

        assertEquals(listOf(":features:status:impl"), UnitDependencyDirection.findForbidden(excepted))
    }

    @Test
    fun GIVEN_host_ui_cycle_WHEN_checked_THEN_boundary_permission_does_not_hide_cycle() {
        val hostRole = UnitRole.ImplUi(composition = UiComposition.HOST)
        val left = appUnit(
            ":root:ui",
            hostRole,
            mapOf(":feature:ui" to hostRole)
        )
        val right = appUnit(
            ":feature:ui",
            hostRole,
            mapOf(":root:ui" to hostRole)
        )

        assertEquals(emptyList(), UnitDependencyDirection.findForbidden(left))
        assertEquals(emptyList(), UnitDependencyDirection.findForbidden(right))
        assertEquals(
            listOf(":feature:ui -> :root:ui -> :feature:ui"),
            UnitDependencyCycle().check(listOf(left, right)).map { violation -> violation.subject }
        )
    }

    @Test
    fun GIVEN_exception_without_declared_edge_WHEN_checked_THEN_reports_configuration_error() {
        val ui = appUnit(
            ":features:rating:ui",
            UnitRole.ImplUi(),
            emptyMap()
        )
        val context = ui.context!!.copy(
            policy = ui.context.policy.copy(
                checks = CheckPolicy(emptyMap(), mapOf(":features:towns:impl" to "temporary migration"))
            )
        )
        val configured = BuildUnit(
            ui.path,
            ui.directory,
            false,
            false,
            emptySet(),
            context = context
        )

        assertFailsWith<IllegalArgumentException> { UnitDependencyDirection.findForbidden(configured) }
    }

    @Test
    fun GIVEN_exception_for_fake_role_WHEN_checked_THEN_rejects_the_target_role() {
        val ui = appUnit(
            ":features:rating:ui",
            UnitRole.ImplUi(),
            mapOf(":features:towns:fake" to UnitRole.Fake())
        )
        val context = ui.context!!.copy(
            policy = ui.context.policy.copy(
                checks = CheckPolicy(emptyMap(), mapOf(":features:towns:fake" to "test fixture"))
            )
        )
        val configured = BuildUnit(
            ui.path,
            ui.directory,
            false,
            false,
            ui.projectDependencies,
            context = context,
            dependencyContexts = ui.dependencyContexts
        )

        assertFailsWith<IllegalArgumentException> { UnitDependencyDirection.findForbidden(configured) }
    }

    @Test
    fun GIVEN_exception_for_entrypoint_role_WHEN_checked_THEN_rejects_the_target_role() {
        val unit = appUnit(
            ":feature:ui",
            UnitRole.ImplUi(),
            mapOf(":app" to UnitRole.EntryPoint())
        )
        val context = unit.context!!.copy(
            policy = unit.context.policy.copy(
                checks = CheckPolicy(emptyMap(), mapOf(":app" to "legacy shortcut"))
            )
        )
        val configured = BuildUnit(
            unit.path,
            unit.directory,
            false,
            false,
            unit.projectDependencies,
            context = context,
            dependencyContexts = unit.dependencyContexts
        )

        assertFailsWith<IllegalArgumentException> { UnitDependencyDirection.findForbidden(configured) }
    }

    @Test
    fun GIVEN_fake_source_allowance_WHEN_checked_THEN_rejects_the_ineffective_exception() {
        val fake = appUnit(
            ":feature:fake",
            UnitRole.Fake(),
            mapOf(":other:api" to UnitRole.Api())
        )
        val context = fake.context!!.copy(
            policy = fake.context.policy.copy(
                checks = CheckPolicy(emptyMap(), mapOf(":other:api" to "legacy fake use"))
            )
        )
        val configured = BuildUnit(
            fake.path,
            fake.directory,
            false,
            false,
            fake.projectDependencies,
            context = context,
            dependencyContexts = fake.dependencyContexts
        )

        assertFailsWith<IllegalArgumentException> { UnitDependencyDirection.findForbidden(configured) }
    }

    @Test
    fun GIVEN_documented_unit_dependencies_WHEN_classified_THEN_accepts_only_allowed_targets() {
        val allowed = mapOf(
            ":instances:bukkit" to ":feature:event-bukkit",
            ":gui:bukkit" to ":core:bukkit",
            ":event-bukkit" to ":data:contract",
            ":gui:api" to ":data:dao",
            ":jail:impl" to ":economy:api",
            ":data:exposed" to ":data:contract",
            ":data:dao" to ":core",
            ":core" to ":",
            ":core:bukkit" to ":core:api",
            ":core:ui" to ":core:api",
            ":payment:impl" to ":core:ui",
            ":dictionary:impl" to ":dictionary:contract",
            ":jail:fake" to ":jail:api"
        )

        allowed.forEach { (path, dependency) ->
            val dependencies = if (dependency == ":") emptySet() else setOf(dependency)
            assertEquals(
                emptyList(),
                forbidden(
                    path,
                    dependencies,
                    isEntryPoint = path == ":instances:bukkit"
                ),
                path
            )
        }
    }

    @Test
    fun GIVEN_dependencies_outside_the_direction_table_WHEN_classified_THEN_reports_each() {
        val forbidden = mapOf(
            ":gui:bukkit" to ":event-bukkit",
            ":event-bukkit" to ":gui:bukkit",
            ":gui:api" to ":core:bukkit",
            ":jail:impl" to ":event-bukkit",
            ":data:exposed" to ":other:room",
            ":data:dao" to ":jail:api",
            ":dictionary:contract" to ":core",
            ":core" to ":jail:api",
            ":core:bukkit" to ":jail:api",
            ":core:ui" to ":core:bukkit",
            ":payment:api" to ":core:ui",
            ":jail:fake" to ":economy:api"
        )

        forbidden.forEach { (path, dependency) ->
            assertEquals(
                listOf(dependency),
                forbidden(path, setOf(dependency)),
                path
            )
        }
    }

    @Test
    fun GIVEN_published_library_dependencies_WHEN_classified_THEN_accepts_them() {
        assertEquals(
            emptyList(),
            forbidden(
                ":gradle-plugin",
                setOf(":detekt", ":structure"),
                isLibrary = true
            )
        )
    }

    @Test
    fun GIVEN_core_unit_with_strict_explicit_api_WHEN_classified_THEN_still_depends_on_no_feature() {
        assertEquals(
            listOf(":gui:api"),
            forbidden(
                ":core:api",
                setOf(":gui:api"),
                isLibrary = true
            )
        )
    }

    @Test
    fun GIVEN_product_platform_adapters_WHEN_classified_THEN_accepts_their_contracts_and_platform_core() {
        val messenger = explicitUnit(
            ":modules:messenger:bukkit",
            UnitKind.PLATFORM,
            mapOf(
                ":modules:messenger:api" to UnitKind.API,
                ":modules:core:api" to UnitKind.CORE_API,
                ":modules:core:bukkit" to UnitKind.CORE_PLATFORM
            )
        )
        val soulService = explicitUnit(
            ":modules:service:forge",
            UnitKind.PLATFORM,
            mapOf(
                ":modules:service:api" to UnitKind.API,
                ":modules:data:dao" to UnitKind.STORAGE_CONTRACT,
                ":modules:core:forge" to UnitKind.CORE_PLATFORM
            )
        )

        assertEquals(emptyList(), UnitDependencyDirection.findForbidden(messenger))
        assertEquals(emptyList(), UnitDependencyDirection.findForbidden(soulService))
    }

    @Test
    fun GIVEN_platform_and_technology_service_roles_WHEN_classified_THEN_technology_rejects_api_and_platform_core() {
        val corePlatform = explicitUnit(
            ":modules:core:bukkit",
            UnitKind.CORE_PLATFORM,
            mapOf(":modules:core:api" to UnitKind.CORE_API)
        )
        val dependencies = mapOf(
            ":modules:service:api" to UnitKind.API,
            ":modules:data:dao" to UnitKind.STORAGE_CONTRACT,
            ":modules:core:api" to UnitKind.CORE_API,
            ":modules:core:forge" to UnitKind.CORE_PLATFORM
        )
        val platform = explicitUnit(
            ":modules:service:forge",
            UnitKind.PLATFORM,
            dependencies
        )
        val technology = explicitUnit(
            ":modules:service:forge",
            UnitKind.TECHNOLOGY,
            dependencies
        )

        assertEquals(emptyList(), UnitDependencyDirection.findForbidden(corePlatform))
        assertEquals(emptyList(), UnitDependencyDirection.findForbidden(platform))
        assertEquals(
            listOf(":modules:core:forge", ":modules:service:api"),
            UnitDependencyDirection.findForbidden(technology)
        )
    }

    @Test
    fun GIVEN_explicit_roles_on_conventional_paths_WHEN_classified_THEN_explicit_roles_win() {
        val platform = explicitUnit(
            ":modules:service:forge",
            UnitKind.PLATFORM,
            mapOf(":modules:core-bukkit" to UnitKind.CORE_PLATFORM)
        )
        val corePlatform = explicitUnit(
            ":modules:core-bukkit",
            UnitKind.CORE_PLATFORM,
            mapOf(":modules:gui:api" to UnitKind.API)
        )

        assertEquals(emptyList(), UnitDependencyDirection.findForbidden(platform))
        assertEquals(
            listOf(":modules:core-bukkit"),
            forbidden(":modules:service:forge", setOf(":modules:core-bukkit"))
        )
        assertEquals(listOf(":modules:gui:api"), UnitDependencyDirection.findForbidden(corePlatform))
        assertEquals(emptyList(), forbidden(":modules:core-bukkit", setOf(":modules:gui:api")))
    }

    @Test
    fun GIVEN_published_astralibs_menu_adapter_WHEN_classified_THEN_keeps_library_dependencies() {
        val dependencies = mapOf(
            ":core" to UnitKind.CORE_API,
            ":core-bukkit" to UnitKind.TECHNOLOGY
        )
        val publishedMenuBukkit = explicitUnit(
            ":menu-bukkit",
            UnitKind.TECHNOLOGY,
            dependencies,
            isLibrary = true
        )
        val productTechnology = explicitUnit(
            ":menu-bukkit",
            UnitKind.TECHNOLOGY,
            dependencies
        )

        assertEquals(emptyList(), UnitDependencyDirection.findForbidden(publishedMenuBukkit))
        assertEquals(
            listOf(":core-bukkit"),
            UnitDependencyDirection.findForbidden(productTechnology)
        )
    }

    @Test
    fun GIVEN_design_system_split_into_units_WHEN_classified_THEN_accepts_acyclic_ui_dependencies() {
        val designSystemDependencies = mapOf(
            ":services:core:ui:common" to setOf(
                ":services:core:common",
                ":services:core:resources",
                ":services:core:ui:theme"
            ),
            ":services:core:ui:theme" to setOf(
                ":services:core:common",
                ":services:core:resources"
            ),
            ":services:core:ui:sheet" to setOf(
                ":services:core:common",
                ":services:core:resources",
                ":services:core:ui:theme"
            )
        )

        designSystemDependencies.forEach { (path, dependencies) ->
            assertEquals(
                emptyList(),
                forbidden(path, dependencies),
                path
            )
        }
        assertEquals(
            listOf(":features:rating:api"),
            forbidden(
                ":services:core:ui:common",
                setOf(":features:rating:api")
            )
        )
    }

    @Test
    fun GIVEN_custom_core_ui_units_WHEN_classified_THEN_allow_ui_and_reject_other_roles() {
        val components = explicitUnit(
            ":design:components",
            UnitKind.CORE_UI,
            mapOf(
                ":design:tokens" to UnitKind.CORE_UI,
                ":services:settings" to UnitKind.CORE_API,
                ":features:rating:api" to UnitKind.API,
                ":platform:android" to UnitKind.CORE_PLATFORM
            )
        )

        assertEquals(
            listOf(":features:rating:api", ":platform:android"),
            UnitDependencyDirection.findForbidden(components)
        )
    }

    @Test
    fun GIVEN_explicit_roles_on_custom_paths_WHEN_classified_THEN_uses_context_instead_of_names() {
        val api = UnitContext(
            UnitKind.API,
            ProductKind.APP,
            false,
            emptySet(),
            emptyList()
        )
        val technology = api.copy(kind = UnitKind.TECHNOLOGY)
        val unit = BuildUnit(
            path = ":feature:exports",
            directory = File("/project"),
            isEntryPoint = false,
            isRoot = false,
            projectDependencies = setOf(":storage:adapter"),
            context = api,
            dependencyContexts = mapOf(":storage:adapter" to technology)
        )

        assertEquals(listOf(":storage:adapter"), UnitDependencyDirection.findForbidden(unit))
        assertEquals(listOf(":storage:adapter"), ApiUnitDependency().check(unit).map { finding -> finding.subject })
        assertEquals(listOf(":storage:adapter"), TechnologyDependency().check(unit).map { finding -> finding.subject })
    }

    @Test
    fun GIVEN_production_dependency_on_shared_fakes_WHEN_entrypoint_or_library_checked_THEN_rejects_it() {
        assertEquals(
            listOf(":feature:fake"),
            forbidden(
                ":instances:jvm",
                setOf(":feature:fake"),
                isEntryPoint = true
            )
        )
        assertEquals(
            listOf(":feature:fake"),
            forbidden(
                ":published",
                setOf(":feature:fake"),
                isLibrary = true
            )
        )
    }
}
