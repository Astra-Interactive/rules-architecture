@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.check.direction.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.SourceTree
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import java.io.File
import kotlin.test.assertEquals

internal class PackageDependencyCycleTest {
    @Test
    fun GIVEN_real_package_cycle_WHEN_checked_THEN_reports_the_complete_path_once(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main("x.feature.a/A.kt", "import x.feature.b.B\nclass A(val b: B)")
            .main("x.feature.b/B.kt", "class B(val c: x.feature.c.C)")
            .main("x.feature.c/C.kt", "import x.feature.a.*\nclass C(val a: A)")
            .load(UnitKind.CONTEXT)

        val findings = PackageDependencyCycle().check(sources)

        assertEquals(
            listOf("x.feature.a -> x.feature.b -> x.feature.c -> x.feature.a"),
            findings.map(Violation::subject)
        )
        assertEquals("src/main/kotlin/x/feature/a/A.kt", findings.single().path)
    }

    @Test
    fun GIVEN_dag_with_external_and_same_package_references_WHEN_checked_THEN_accepts_it(@TempDir directory: File) {
        val sources = SourceTree(directory)
            .main(
                "x.feature.a/A.kt",
                """
                import x.feature.a.Peer
                import x.feature.b.B as Item
                import java.util.*

                class A(val b: Item, val peer: Peer)
                """
            )
            .main("x.feature.a/Peer.kt", "import java.io.File\nclass Peer(val file: File)")
            .main(
                "x.feature.b/B.kt",
                """
                class B
                // x.feature.a.A
                val text = "x.feature.a.A"
                """
            )
            .load(UnitKind.CONTEXT)

        assertEquals(emptyList(), PackageDependencyCycle().check(sources))
    }

    @Test
    fun GIVEN_test_cycle_WHEN_checked_THEN_reports_it_without_making_it_a_production_edge(@TempDir directory: File) {
        val tree = SourceTree(directory)
            .main("x.feature.model/Model.kt", "class Model")
            .test("x.feature.first/First.kt", "import x.feature.second.Second\nclass First(val second: Second)")
            .test("x.feature.second/Second.kt", "import x.feature.first.First\nclass Second(val first: First)")
        val sources = tree.load(UnitKind.CONTEXT)

        assertEquals(
            listOf("x.feature.first -> x.feature.second -> x.feature.first"),
            PackageDependencyCycle().check(sources).map(Violation::subject)
        )
        val production = sources.context.copy(
            compilations = sources.context.compilations.filterNot(UnitContext.Compilation::test)
        )
        assertEquals(emptyList(), PackageDependencyCycle().check(tree.load(UnitKind.CONTEXT, context = production)))
    }

    @Test
    fun GIVEN_disjoint_platform_edges_WHEN_checked_THEN_never_combines_their_packages_into_a_cycle(
        @TempDir directory: File
    ) {
        val tree = SourceTree(directory)
            .main("x.feature.a/A.kt", "class A(val b: x.feature.b.B)")
            .main("x.feature.b/B.kt", "class B(val a: x.feature.a.A)")
        val original = tree.load(UnitKind.CONTEXT)
        val a = File(tree.mainDirectory, "x/feature/a").absolutePath
        val b = File(tree.mainDirectory, "x/feature/b").absolutePath
        val compilations = listOf("androidMain", "iosMain").map { name ->
            UnitContext.Compilation(
                name,
                mapOf(name to setOf(if (name == "androidMain") a else b)),
                false,
                false,
                emptySet()
            )
        }
        val context = original.context.copy(compilations = compilations)

        assertEquals(emptyList(), PackageDependencyCycle().check(tree.load(UnitKind.CONTEXT, context = context)))
        assertEquals(1, PackageDependencyCycle().check(tree.load(UnitKind.CONTEXT, context = null)).size)
    }

    @Test
    fun GIVEN_empty_sources_WHEN_checked_THEN_reports_nothing(@TempDir directory: File) {
        assertEquals(emptyList(), PackageDependencyCycle().check(SourceTree(directory).load(UnitKind.CONTEXT)))
    }
}
