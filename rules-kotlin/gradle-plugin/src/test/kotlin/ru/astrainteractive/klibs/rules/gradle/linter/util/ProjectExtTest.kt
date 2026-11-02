@file:Suppress("FunctionNaming")

package ru.astrainteractive.klibs.rules.gradle.linter.util

import org.gradle.testfixtures.ProjectBuilder
import org.jetbrains.kotlin.gradle.dsl.KotlinMultiplatformExtension
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.io.TempDir
import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.util.toUnitContext
import ru.astrainteractive.klibs.rules.gradle.fixture.IsolatedProjectsBuild
import java.io.File

internal class ProjectExtTest {
    @Test
    fun GIVEN_custom_depends_on_and_test_source_sets_WHEN_read_THEN_preserves_declaration_and_consumers(
        @TempDir rootDirectory: File
    ) {
        val project = ProjectBuilder.builder().withProjectDir(rootDirectory).build()
        project.pluginManager.apply("org.jetbrains.kotlin.multiplatform")
        val kotlin = project.extensions.getByType(KotlinMultiplatformExtension::class.java)
        kotlin.jvm("desktop")
        val shared = kotlin.sourceSets.create("sharedMain")
        kotlin.sourceSets.getByName("desktopMain").dependsOn(shared)
        val common = kotlin.sourceSets.getByName("commonMain")
        val test = kotlin.sourceSets.getByName("desktopTest")
        project.dependencies.add(shared.implementationConfigurationName, "androidx.core:core-ktx:1.0")
        project.dependencies.add(common.implementationConfigurationName, "androidx.core:core-splashscreen:1.0")
        project.dependencies.add(test.implementationConfigurationName, "androidx.core:core-ktx:1.0")

        val declarations = project.readUnitBuild(UnitKind.CORE, ProductKind.APP).externalDependencies

        val custom = declarations.single { declaration -> declaration.sourceSet == "sharedMain" }
        assertEquals("sharedMainImplementation", custom.configuration)
        assertEquals(true, custom.production)
        assertEquals(setOf("desktop"), custom.targetCompilations.map { use -> use.target }.toSet())
        assertEquals(setOf("main"), custom.targetCompilations.map { use -> use.compilation }.toSet())
        val commonDeclaration = declarations.single { declaration -> declaration.sourceSet == "commonMain" }
        assertEquals(true, commonDeclaration.production)
        assertEquals("commonMain", commonDeclaration.sourceSet)
        val testDeclaration = declarations.single { declaration -> declaration.sourceSet == "desktopTest" }
        assertEquals(false, testDeclaration.production)
        assertEquals(true, testDeclaration.test)
    }

    private fun unitKindScript(plugins: String): String {
        return """
            plugins {
                $plugins
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }

            tasks.register("reportUnitKind") {
                dependsOn("architectureDetektSettings")
            }
        """.trimIndent()
    }

    private fun readContext(build: IsolatedProjectsBuild, module: String): UnitContext {
        val path = module.replace(':', '/')
        val encoded = build.file("$path/build/detekt/unit-context.txt").readText()
        return encoded.toUnitContext()
    }

    @Test
    fun GIVEN_conflicting_enum_roles_WHEN_configured_THEN_reports_both_declarations(@TempDir rootDirectory: File) {
        val script = """
            import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind

            $JAVA_MODULE_SCRIPT

            linter {
                unitKind(UnitKind.API)
                unitKind(UnitKind.CONTEXT)
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .withRootBuildScript("")
            .withModule(name = "modules:offer", buildScript = script)

        val result = build.runAndFail("help")

        assertTrue(result.output.contains("Conflicting linter.unitKind declarations: API and CONTEXT"), result.output)
    }

    @Test
    fun GIVEN_conflicting_product_kinds_WHEN_configured_THEN_reports_the_conflict(@TempDir rootDirectory: File) {
        val script = """
            import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind

            $JAVA_MODULE_SCRIPT

            linter {
                productKind(ProductKind.APP)
                productKind(ProductKind.PLATFORM_PLUGIN)
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .withRootBuildScript("")
            .withModule(name = "modules:offer", buildScript = script)

        val result = build.runAndFail("help")

        assertTrue(result.output.contains("Conflicting linter.productKind declarations"), result.output)
    }

    @Test
    fun GIVEN_app_product_on_root_WHEN_api_checked_twice_THEN_inherits_it_and_reuses_configuration_cache(
        @TempDir rootDirectory: File
    ) {
        val rootScript = """
            import ru.astrainteractive.klibs.rules.core.unit.model.ProductKind

            plugins {
                base
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }
            linter { productKind(ProductKind.APP) }
        """.trimIndent()
        val apiScript = """
            import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind

            $JAVA_MODULE_SCRIPT

            linter {
                unitKind(UnitKind.API)
                unitKind(UnitKind.API)
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript(rootScript)
            .withModule(name = "modules:offer:api", buildScript = apiScript)
            .withSourceFile("modules/offer/api/README.md", "Declares the offer capability used by other units.")
            .withSourceFile(
                "modules/offer/api/src/main/kotlin/x/offer/api/offer/usecase/BuyOfferUseCase.kt",
                """
                package x.offer.api.offer.usecase

                interface BuyOfferUseCase {
                    operator fun invoke()
                }
                """.trimIndent()
            )

        build.run(":modules:offer:api:architectureCheck")
        val second = build.run(":modules:offer:api:architectureCheck")

        assertTrue(second.output.contains("Reusing configuration cache"), second.output)
    }

    @Test
    fun GIVEN_api_usecase_without_product_kind_WHEN_checked_THEN_requests_the_required_enum(
        @TempDir rootDirectory: File
    ) {
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(name = "modules:offer:api", buildScript = JAVA_MODULE_SCRIPT)
            .withSourceFile("modules/offer/api/README.md", "Declares the offer capability used by other units.")
            .withSourceFile(
                "modules/offer/api/src/main/kotlin/x/offer/api/offer/usecase/BuyOfferUseCase.kt",
                "package x.offer.api.offer.usecase\ninterface BuyOfferUseCase { operator fun invoke() }"
            )

        val result = build.runAndFail(":modules:offer:api:architectureCheck")

        assertTrue(result.output.contains("Set linter.productKind"), result.output)
    }

    @Test
    fun GIVEN_entry_point_conventions_and_explicit_roles_WHEN_configured_THEN_explicit_roles_win(
        @TempDir rootDirectory: File
    ) {
        val inferred = unitKindScript("`java-library`")
        val explicitTechnology = inferred + """

            linter { unitKind(ru.astrainteractive.klibs.rules.core.unit.model.UnitKind.TECHNOLOGY) }
        """.trimIndent()
        val explicitEntryPoint = inferred + """

            linter { unitKind(ru.astrainteractive.klibs.rules.core.unit.model.UnitKind.ENTRY_POINT) }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .withRootBuildScript("")
            .withModule(name = "instances:forge", buildScript = inferred)
            .withModule(name = "instances:neoforge", buildScript = inferred)
            .withModule(name = "instances:core:api", buildScript = inferred)
            .withModule(name = "instances:exposed", buildScript = explicitTechnology)
            .withModule(name = "exposed", buildScript = explicitEntryPoint)
            .withModule(name = "memory", buildScript = unitKindScript("application"))
        val modules = listOf(
            "instances:forge",
            "instances:neoforge",
            "instances:core:api",
            "instances:exposed",
            "exposed",
            "memory"
        )

        build.run(*modules.map { module -> ":$module:reportUnitKind" }.toTypedArray())

        assertEquals(
            listOf(
                "entry-point",
                "entry-point",
                "entry-point",
                "technology",
                "entry-point",
                "entry-point"
            ),
            modules.map { module -> readContext(build, module).kind.cliName }
        )
    }

    @Test
    fun GIVEN_units_named_and_built_by_convention_WHEN_configured_THEN_detects_their_kind(
        @TempDir rootDirectory: File
    ) {
        val build = IsolatedProjectsBuild(rootDirectory)
            .withRootBuildScript("")
            .withModule(name = "api", buildScript = unitKindScript("`java-library`"))
            .withModule(name = "contract", buildScript = unitKindScript("`java-library`"))
            .withModule(name = "server", buildScript = unitKindScript("application"))
            .withModule(name = "exposed", buildScript = unitKindScript("`java-library`"))
            .withModule(name = "memory", buildScript = unitKindScript("`java-library`"))
            .withModule(name = "jail", buildScript = unitKindScript("`java-library`"))
            .withModule(name = "gui:bukkit", buildScript = unitKindScript("`java-library`"))
            .withModule(name = "core:bukkit", buildScript = unitKindScript("`java-library`"))
            .withModule(name = "instances:bukkit", buildScript = unitKindScript("`java-library`"))
        val modules = listOf(
            "api",
            "contract",
            "server",
            "exposed",
            "memory",
            "jail",
            "gui:bukkit",
            "core:bukkit",
            "instances:bukkit"
        )

        build.run(*modules.map { module -> ":$module:reportUnitKind" }.toTypedArray())

        val kinds = modules.map { module ->
            readContext(build, module).kind.cliName
        }
        assertEquals(
            listOf(
                "api",
                "contract",
                "entry-point",
                "technology",
                "technology",
                "context",
                "platform",
                "core-platform",
                "entry-point"
            ),
            kinds
        )
    }

    @Test
    fun GIVEN_kotlin_units_with_explicit_api_WHEN_configured_THEN_only_the_published_one_is_a_library(
        @TempDir rootDirectory: File
    ) {
        val explicitApi = """

            kotlin {
                explicitApi()
            }
        """.trimIndent()
        val kotlinPlugin = """id("org.jetbrains.kotlin.jvm")"""
        val build = IsolatedProjectsBuild(rootDirectory)
            .withRootBuildScript("")
            .withModule(
                name = "published",
                buildScript = unitKindScript("$kotlinPlugin; `maven-publish`") + explicitApi
            )
            .withModule(
                name = "jail",
                buildScript = unitKindScript(kotlinPlugin) + explicitApi
            )

        build.run(":published:reportUnitKind", ":jail:reportUnitKind")

        assertEquals("context", readContext(build, "published").kind.cliName)
        assertEquals(true, readContext(build, "published").publishedLibrary)
        assertEquals(false, readContext(build, "jail").publishedLibrary)
        assertEquals("context", readContext(build, "jail").kind.cliName)
    }

    @Test
    fun GIVEN_compose_dependencies_WHEN_context_read_THEN_uses_each_real_compilation(@TempDir rootDirectory: File) {
        val jvm = unitKindScript("""id("org.jetbrains.kotlin.jvm")""")
        val multiplatform = unitKindScript("""id("org.jetbrains.kotlin.multiplatform")""") + """

            kotlin {
                jvm()
                sourceSets.commonMain.dependencies {
                    implementation("org.jetbrains.compose.runtime:runtime:1.8.2")
                }
            }
        """.trimIndent()
        val build = IsolatedProjectsBuild(rootDirectory)
            .online()
            .withRootBuildScript("")
            .withModule(
                name = "production",
                buildScript = jvm + """

                    dependencies { implementation("org.jetbrains.compose.runtime:runtime:1.8.2") }
                """.trimIndent()
            )
            .withModule(
                name = "testonly",
                buildScript = jvm + """

                    dependencies { testImplementation("org.jetbrains.compose.runtime:runtime:1.8.2") }
                """.trimIndent()
            )
            .withModule(name = "multiplatform", buildScript = multiplatform)
            .withModule(name = "importonly", buildScript = jvm)
            .withSourceFile(
                "importonly/src/main/kotlin/Example.kt",
                "import androidx.compose.runtime.Composable\n@Composable fun Example() = Unit"
            )

        build.run(
            ":production:reportUnitKind",
            ":testonly:reportUnitKind",
            ":multiplatform:reportUnitKind",
            ":importonly:reportUnitKind"
        )

        val production = readContext(build, "production").compilations
        val testOnly = readContext(build, "testonly").compilations
        val common = readContext(build, "multiplatform").compilations
        assertTrue(production.all { compilation -> compilation.compose == true })
        val testOnlyProduction = testOnly.filterNot(UnitContext.Compilation::test)
        assertTrue(testOnlyProduction.none { compilation -> compilation.compose == true })
        assertTrue(testOnly.filter(UnitContext.Compilation::test).all { compilation -> compilation.compose == true })
        assertTrue(common.filterNot(UnitContext.Compilation::test).all { compilation -> compilation.compose == true })
        assertTrue(common.any { compilation -> "commonMain" in compilation.sourceSets })
        assertTrue(readContext(build, "importonly").compilations.none { compilation -> compilation.compose == true })
    }

    private companion object {
        val JAVA_MODULE_SCRIPT = """
            plugins {
                `java-library`
                id("ru.astrainteractive.klibs.rules.gradle.linter")
            }
        """.trimIndent()
    }
}
