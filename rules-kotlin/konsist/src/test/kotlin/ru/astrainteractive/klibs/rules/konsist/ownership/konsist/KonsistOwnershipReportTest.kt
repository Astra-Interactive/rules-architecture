@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.konsist.ownership.konsist

import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.util.encode
import ru.astrainteractive.klibs.rules.konsist.ownership.model.OwnershipUnit
import java.io.File
import kotlin.test.assertContains
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse

class KonsistOwnershipReportTest {
    @TempDir
    lateinit var root: File

    private fun unit(
        path: String,
        kind: UnitKind,
        published: Boolean = false,
        sourceSets: Set<String> = setOf("main"),
        dependencies: Set<String> = emptySet()
    ): OwnershipUnit {
        val directory = File(root, path.replace(':', '/').trimStart('/'))
        val main = sourceSets.associateWith { name -> setOf(File(directory, "src/$name/kotlin").absolutePath) }
        val test = mapOf("test" to setOf(File(directory, "src/test/kotlin").absolutePath))
        val context = UnitContext(
            kind,
            null,
            published,
            emptySet(),
            listOf(
                UnitContext.Compilation(
                    "main",
                    main,
                    false,
                    null,
                    dependencies
                ),
                UnitContext.Compilation(
                    "test",
                    test,
                    true,
                    null,
                    emptySet()
                )
            )
        )
        return OwnershipUnit(
            path,
            directory,
            context
        )
    }

    private fun OwnershipUnit.write(name: String, text: String, sourceSet: String = "main") {
        val file = File(directory, "src/$sourceSet/kotlin/$name.kt")
        file.parentFile.mkdirs()
        file.writeText(text.trimIndent())
    }

    @Test
    fun GIVEN_alias_wildcard_fully_qualified_nested_and_same_package_uses_WHEN_scanned_THEN_reports_evidence() {
        val api = unit(":voice:api", UnitKind.API)
        val impl = unit(":voice:jda", UnitKind.PLATFORM)
        api.write(
            "Models",
            """
            package x
            class Aliased
            class Wildcard
            class Qualified
            class SamePackage
            class Parent { class Child }
            class UnusedImport
        """
        )
        impl.write(
            "Alias",
            """
            package y
            import x.Aliased as Renamed
            import x.Parent.Child
            import x.UnusedImport
            import java.time.Instant
            class Caller(val value: Renamed, val child: Child, val qualified: x.Qualified, val instant: Instant)
        """
        )
        impl.write("Wildcard", "package z\nimport x.*\nfun consume(value: Wildcard) = value")
        impl.write("Same", "package x\nclass Same(val value: SamePackage)")

        val report = KonsistOwnershipReport().render(listOf(api, impl))

        listOf(
            "Aliased",
            "Wildcard",
            "Qualified",
            "Parent",
            "SamePackage"
        ).forEach { name ->
            assertContains(report, "CANDIDATE x.$name")
        }
        assertContains(report, "UNKNOWN x.UnusedImport")
        assertContains(report, "REFERENCE :voice:jda src/main/kotlin/Alias.kt:6")
    }

    @Test
    fun GIVEN_comments_literals_tests_and_di_WHEN_scanned_THEN_does_not_claim_production_consumers() {
        val api = unit(":voice:api", UnitKind.API)
        val impl = unit(":voice:jda", UnitKind.PLATFORM)
        api.write("Local", "package x\nclass Local")
        impl.write(
            "Noise",
            """
            package y
            import x.Local
            // Local in a comment
            /* Local /* nested Local */ comment */
            val name = "Local"
        """
        )
        impl.write(
            "Test",
            "package y\nimport x.Local\nclass Test(val local: Local)",
            "test"
        )
        impl.write(
            "Fake",
            "package y\nimport x.Local\nclass Fake(val local: Local)",
            "testFixtures"
        )
        impl.write("Module", "package y.di\nimport x.Local\nclass Module { val local = Local() }")

        val report = KonsistOwnershipReport().render(listOf(api, impl))

        assertContains(report, "UNKNOWN x.Local")
        assertContains(report, "DI :voice:jda")
        assertFalse(report.contains("CANDIDATE x.Local"))
        assertFalse(report.contains("REFERENCE :voice:jda"))
    }

    @Test
    fun GIVEN_repository_signature_generics_aliases_defaults_and_nested_types_WHEN_shared_THEN_retains_closure() {
        val api = unit(":voice:api", UnitKind.API)
        val jda = unit(":voice:jda", UnitKind.PLATFORM)
        val exposed = unit(":voice:exposed", UnitKind.TECHNOLOGY)
        api.write(
            "Repository",
            """
            package x.api
            import x.model.Alias
            import x.model.DefaultValue
            interface Repository {
                fun load(value: Alias = DefaultValue): List<Alias>
            }
        """
        )
        api.write("Alias", "package x.model\ntypealias Alias = Model.Nested")
        api.write("Model", "package x.model\nclass Model { class Nested(val id: Id) }\nclass Id")
        api.write("Default", "package x.model\nval DefaultValue = Model.Nested(Id())")
        jda.write("Caller", "package y\nimport x.api.Repository\nclass Caller(val repository: Repository)")
        exposed.write(
            "Implementation",
            "package z\nimport x.api.Repository\nabstract class Implementation : Repository"
        )

        val report = KonsistOwnershipReport().render(
            listOf(
                api,
                jda,
                exposed
            )
        )

        listOf(
            "x.api.Repository",
            "x.model.Alias",
            "x.model.DefaultValue",
            "x.model.Model",
            "x.model.Id"
        )
            .forEach { name -> assertContains(report, "RETAIN $name") }
        assertFalse(report.contains("CANDIDATE "))
    }

    @Test
    fun GIVEN_kmp_variants_and_custom_production_roots_WHEN_scanned_THEN_merges_all_platform_consumers() {
        val api = unit(
            ":voice:api",
            UnitKind.API,
            sourceSets = setOf("commonMain", "jvmMain")
        )
        val impl = unit(
            ":voice:impl",
            UnitKind.IMPL,
            sourceSets = setOf("androidMain", "desktopMain")
        )
        api.write(
            "Expected",
            "package x\nexpect class Local",
            "commonMain"
        )
        api.write(
            "Actual",
            "package x\nactual class Local",
            "jvmMain"
        )
        impl.write(
            "Android",
            "package y\nclass Android(val local: x.Local)",
            "androidMain"
        )
        impl.write(
            "Desktop",
            "package z\nclass Desktop(val local: x.Local)",
            "desktopMain"
        )

        val report = KonsistOwnershipReport().render(listOf(api, impl))

        assertEquals(1, Regex("CANDIDATE x.Local").findAll(report).count())
        assertContains(report, "src/androidMain/kotlin/Android.kt")
        assertContains(report, "src/desktopMain/kotlin/Desktop.kt")
        assertContains(report, "src/commonMain/kotlin/Expected.kt")
        assertContains(report, "src/jvmMain/kotlin/Actual.kt")
    }

    @Test
    fun GIVEN_published_api_and_missing_metadata_WHEN_scanned_THEN_preserves_external_contract_and_limits() {
        val api = unit(
            ":voice:api",
            UnitKind.API,
            published = true
        )
        val impl = unit(
            ":voice:impl",
            UnitKind.IMPL,
            dependencies = setOf(":missing")
        )
        val unknown = OwnershipUnit(
            ":unknown",
            root,
            UnitContext(
                UnitKind.CONTEXT,
                null,
                false,
                emptySet(),
                emptyList()
            )
        )
        api.write("PublicApi", "package x\nclass PublicApi")

        val report = KonsistOwnershipReport().render(
            listOf(
                api,
                impl,
                unknown
            )
        )

        assertContains(report, "EXTERNAL x.PublicApi")
        assertContains(report, "no Kotlin compilation metadata")
        assertContains(report, "absent source directory")
        assertContains(report, "missing unit metadata for :missing")
        assertContains(report, "published API retained")
    }

    @Test
    fun GIVEN_same_short_names_overloads_and_string_template_WHEN_scanned_THEN_is_conservative() {
        val api = unit(":voice:api", UnitKind.API)
        val impl = unit(":voice:impl", UnitKind.IMPL)
        api.write(
            "One",
            """
            package x
            class Value
            fun build(value: String) = value
            fun build(value: Int) = value
            """
        )
        api.write("Two", "package z\nclass Value")
        impl.write(
            "Caller",
            """
            package y
            import x.*
            import z.*
            fun use(value: Value) = "${'$'}{build(1)}"
        """
        )

        val report = KonsistOwnershipReport().render(listOf(api, impl))

        assertContains(report, "UNCERTAIN x.Value")
        assertContains(report, "UNCERTAIN z.Value")
        assertEquals(1, Regex("UNCERTAIN x.build").findAll(report).count())
        assertContains(report, "overloads and shadowing are not compiler-resolved")
    }

    @Test
    fun GIVEN_cli_metadata_WHEN_report_written_THEN_reports_candidates_without_throwing() {
        val api = unit(":voice:api", UnitKind.API)
        val impl = unit(":voice:impl", UnitKind.IMPL)
        api.write("Local", "class Local")
        impl.write("Caller", "class Caller(val local: Local)")
        val metadata = File(root, "units.tsv")
        metadata.writeText(
            listOf(api, impl).joinToString("\n") { unit ->
                listOf(
                    unit.path,
                    unit.directory.absolutePath,
                    unit.context.encode()
                ).joinToString("\t")
            } + "\n\n"
        )
        val output = File(root, "reports/ownership.txt")

        main(arrayOf(metadata.absolutePath, output.absolutePath))

        assertContains(output.readText(), "CANDIDATE Local")
        assertFailsWith<IllegalArgumentException> { main(emptyArray()) }
        metadata.writeText("broken\tmetadata")
        assertFailsWith<IllegalArgumentException> { main(arrayOf(metadata.absolutePath, output.absolutePath)) }
    }
}
