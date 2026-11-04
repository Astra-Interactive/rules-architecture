@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.core.unit.util

import org.junit.jupiter.api.Test
import ru.astrainteractive.klibs.rules.core.unit.model.CheckPolicy
import ru.astrainteractive.klibs.rules.core.unit.model.ImplementationAccess
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UiComposition
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitPolicy
import java.io.ByteArrayOutputStream
import java.io.DataOutputStream
import java.io.File
import java.util.Base64
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

internal class UnitContextExtTest {
    @Test
    fun GIVEN_v2_context_WHEN_decoded_THEN_uses_legacy_policy_defaults() {
        val bytes = ByteArrayOutputStream()
        DataOutputStream(bytes).use { output ->
            output.writeInt(2)
            output.writeUTF("api")
            output.writeUTF("APP")
            output.writeBoolean(false)
            output.writeInt(0)
            output.writeInt(0)
        }

        val decoded = Base64.getEncoder().encodeToString(bytes.toByteArray()).toUnitContext()

        assertEquals(UnitKind.API, decoded.kind)
        assertEquals(
            UnitPolicy(
                UiComposition.LOCAL,
                ImplementationAccess.NONE,
                CheckPolicy(emptyMap(), emptyMap())
            ),
            decoded.policy
        )
    }

    @Test
    fun GIVEN_v3_policy_with_checks_WHEN_encoded_THEN_round_trips_and_orders_maps() {
        val first = UnitContext(
            UnitKind.IMPL_UI,
            ProductKind.APP,
            false,
            emptySet(),
            emptyList(),
            UnitPolicy(
                UiComposition.HOST,
                ImplementationAccess.NONE,
                CheckPolicy(
                    linkedMapOf("UnitDependency" to "legacy edge", "ApiUnitDependency" to "migration"),
                    linkedMapOf(":z:impl" to "old split", ":a:impl" to "migration")
                )
            )
        )
        val reversed = first.copy(
            policy = first.policy.copy(
                checks = CheckPolicy(
                    first.policy.checks.disabledRules.toList().reversed().toMap(),
                    first.policy.checks.allowedDependencies.toList().reversed().toMap()
                )
            )
        )

        assertEquals(first, first.encode().toUnitContext())
        assertEquals(first.encode(), reversed.encode())
    }

    @Test
    fun GIVEN_v3_context_WHEN_decoded_THEN_compiled_sources_default_to_visible_sources() {
        val bytes = ByteArrayOutputStream()
        DataOutputStream(bytes).use { output ->
            output.writeInt(3)
            output.writeUTF(UnitKind.API.cliName)
            output.writeUTF(ProductKind.APP.name)
            output.writeBoolean(false)
            output.writeInt(0)
            output.writeInt(1)
            output.writeUTF("jvm:test")
            output.writeBoolean(true)
            output.writeUTF("false")
            output.writeInt(0)
            output.writeInt(1)
            output.writeUTF("main")
            output.writeInt(1)
            output.writeUTF("/project/src/main/kotlin")
            output.writeUTF(UiComposition.LOCAL.name)
            output.writeUTF(ImplementationAccess.NONE.name)
            output.writeInt(0)
            output.writeInt(0)
        }

        val compilation = Base64.getEncoder().encodeToString(bytes.toByteArray()).toUnitContext().compilations.single()

        assertEquals(compilation.sourceSets, compilation.compiledSourceSets)
    }

    @Test
    fun GIVEN_distinct_compiled_sources_WHEN_encoded_THEN_round_trips_both_source_views() {
        val visible = mapOf("main" to setOf("/project/src/main/kotlin"), "test" to setOf("/project/src/test/kotlin"))
        val compiled = mapOf("test" to setOf("/project/src/test/kotlin"))
        val context = UnitContext(
            UnitKind.API,
            ProductKind.APP,
            false,
            emptySet(),
            listOf(
                UnitContext.Compilation(
                    "jvm:test",
                    visible,
                    true,
                    false,
                    emptySet(),
                    compiled
                )
            )
        )

        assertEquals(context, context.encode().toUnitContext())
    }

    @Test
    fun GIVEN_shared_sources_WHEN_selecting_context_THEN_uses_only_compilations_containing_the_file() {
        val root = File("project").absoluteFile
        val common = File(root, "src/commonMain/kotlin")
        val android = File(root, "src/androidMain/kotlin")
        val jvm = File(root, "src/jvmMain/kotlin")
        val context = UnitContext(
            UnitKind.API,
            ProductKind.APP,
            false,
            emptySet(),
            listOf(
                UnitContext.Compilation(
                    "android:main",
                    mapOf("commonMain" to setOf(common.path), "androidMain" to setOf(android.path)),
                    false,
                    true,
                    emptySet()
                ),
                UnitContext.Compilation(
                    "jvm:main",
                    mapOf("commonMain" to setOf(common.path), "jvmMain" to setOf(jvm.path)),
                    false,
                    false,
                    emptySet()
                )
            )
        )

        assertEquals(2, context.compilationsFor(File(common, "State.kt")).size)
        val selectedNames = context.compilationsFor(File(android, "Screen.kt")).map(UnitContext.Compilation::name)
        assertEquals(listOf("android:main"), selectedNames)
        assertTrue(context.isCompose(File(android, "Screen.kt")) == true)
        assertFalse(context.isCompose(File(common, "State.kt")) == true)
        assertFalse(context.isCompose(File(jvm, "State.kt")) == true)
        assertNull(context.isCompose(File(root, "src/androidMain/kotlinExtra/Other.kt")))
    }

    @Test
    fun GIVEN_test_compilation_sees_main_sources_WHEN_selecting_compose_THEN_only_main_compilation_owns_them() {
        val main = File("project/src/main/kotlin").absoluteFile
        val test = File("project/src/test/kotlin").absoluteFile
        val context = UnitContext(
            UnitKind.PLATFORM,
            ProductKind.APP,
            false,
            emptySet(),
            listOf(
                UnitContext.Compilation(
                    "jvm:main",
                    mapOf("main" to setOf(main.path)),
                    false,
                    true,
                    emptySet()
                ),
                UnitContext.Compilation(
                    "jvm:test",
                    mapOf("main" to setOf(main.path), "test" to setOf(test.path)),
                    true,
                    false,
                    emptySet(),
                    mapOf("test" to setOf(test.path))
                )
            )
        )

        assertTrue(context.isCompose(File(main, "PreviewPage.kt")) == true)
        assertFalse(context.isCompose(File(test, "PreviewPageTest.kt")) == true)
    }

    @Test
    fun GIVEN_unusual_paths_and_both_product_kinds_WHEN_transported_THEN_preserves_the_context() {
        ProductKind.entries.forEach { product ->
            val context = UnitContext(
                UnitKind.CONTRACT,
                product,
                true,
                setOf("mongo", "compose"),
                listOf(
                    UnitContext.Compilation(
                        "jvm:test",
                        mapOf("test" to setOf("/project's path/тест\tfiles", "/other\nsource")),
                        true,
                        true,
                        setOf(":modules:fake", ":modules:api")
                    )
                )
            )

            assertEquals(context, context.encode().toUnitContext())
        }
    }

    @Test
    fun GIVEN_equivalent_contexts_in_different_orders_WHEN_encoded_THEN_has_stable_task_inputs() {
        val first = UnitContext.Compilation(
            "jvm:main",
            linkedMapOf("main" to linkedSetOf("/b", "/a"), "shared" to emptySet()),
            false,
            false,
            linkedSetOf(":b", ":a")
        )
        val second = UnitContext.Compilation(
            "jvm:test",
            emptyMap(),
            true,
            false,
            emptySet()
        )
        val context = UnitContext(
            UnitKind.CONTEXT,
            null,
            false,
            emptySet(),
            listOf(first, second)
        )
        val reversed = context.copy(
            compilations = listOf(
                second,
                first.copy(
                    sourceSets = buildMap {
                        first.sourceSets.entries.reversed().forEach { (name, directories) ->
                            put(name, directories.reversed().toSet())
                        }
                    },
                    projectDependencies = first.projectDependencies.reversed().toSet()
                )
            )
        )

        assertEquals(context.encode(), reversed.encode())
        assertEquals(context, context.encode().toUnitContext())
    }

    @Test
    fun GIVEN_unresolved_compose_capability_WHEN_transported_THEN_preserves_unknown_instead_of_false() {
        val root = File("project/src/main/kotlin").absoluteFile
        val compilation = UnitContext.Compilation(
            "jvm:main",
            mapOf("main" to setOf(root.path)),
            false,
            null,
            setOf(":feature:api")
        )
        val context = UnitContext(
            UnitKind.CONTEXT,
            ProductKind.APP,
            false,
            emptySet(),
            listOf(compilation)
        )

        assertEquals(context, context.encode().toUnitContext())
        assertNull(context.isCompose(File(root, "Screen.kt")))
        val mixed = context.copy(compilations = listOf(compilation, compilation.copy(name = "other", compose = false)))
        assertEquals(false, mixed.isCompose(File(root, "Screen.kt")))
    }
}
