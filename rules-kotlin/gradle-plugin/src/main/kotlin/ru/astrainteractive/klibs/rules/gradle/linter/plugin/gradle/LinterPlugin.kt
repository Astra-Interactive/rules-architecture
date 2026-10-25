package ru.astrainteractive.klibs.rules.gradle.linter.plugin.gradle

import dev.detekt.gradle.Detekt
import org.gradle.api.GradleException
import org.gradle.api.NamedDomainObjectProvider
import org.gradle.api.Plugin
import org.gradle.api.Project
import org.gradle.api.artifacts.ResolvableConfiguration
import org.gradle.api.attributes.Bundling
import org.gradle.api.attributes.Category
import org.gradle.api.attributes.LibraryElements
import org.gradle.api.attributes.Usage
import org.gradle.api.attributes.java.TargetJvmEnvironment
import org.gradle.api.file.FileCollection
import org.gradle.api.provider.Provider
import org.gradle.language.base.plugins.LifecycleBasePlugin
import ru.astrainteractive.klibs.rules.gradle.detekt.plugin.gradle.DetektPlugin
import ru.astrainteractive.klibs.rules.gradle.detekt.service.DetektConfigService
import ru.astrainteractive.klibs.rules.gradle.linter.extension.model.LinterExtension
import ru.astrainteractive.klibs.rules.gradle.linter.model.ArchitectureFinding
import ru.astrainteractive.klibs.rules.gradle.linter.model.UnitKind
import ru.astrainteractive.klibs.rules.gradle.linter.service.ArchitectureGraphService
import ru.astrainteractive.klibs.rules.gradle.linter.task.gradle.ArchitectureBaselineTask
import ru.astrainteractive.klibs.rules.gradle.linter.task.gradle.ArchitectureCheckTask
import ru.astrainteractive.klibs.rules.gradle.linter.task.gradle.ArchitectureGraphTask
import ru.astrainteractive.klibs.rules.gradle.linter.task.gradle.ArchitectureTask
import ru.astrainteractive.klibs.rules.gradle.linter.util.detectUnitKind
import ru.astrainteractive.klibs.rules.gradle.linter.util.explicitApiMode
import ru.astrainteractive.klibs.rules.gradle.linter.util.findBuildFindings
import ru.astrainteractive.klibs.rules.gradle.linter.util.productionProjectDependencies
import ru.astrainteractive.klibs.rules.gradle.linter.util.readUnitBuild
import ru.astrainteractive.klibs.rules.gradle.linter.util.toLine
import ru.astrainteractive.klibs.rules.gradle.util.hasKotlinPlugin
import java.util.Properties

internal class LinterPlugin : Plugin<Project> {
    private fun readResource(name: String): String {
        val stream = javaClass.classLoader.getResourceAsStream(name)
            ?: throw GradleException("$name is missing from the gradle plugin jar")
        return stream.use { input -> input.readBytes().decodeToString() }
    }

    private fun readRulesVersion(): String {
        val properties = Properties()
        properties.load(readResource(RULES_VERSION_FILE).reader())
        return properties.getProperty("version")
    }

    private fun Project.explicitApiArguments(): List<String> {
        if (!hasKotlinPlugin()) return emptyList()
        val mode = explicitApiMode() ?: return emptyList()
        return listOf("-Xexplicit-api=$mode")
    }

    private fun Project.configureDetekt(rulesVersion: String) {
        dependencies.add("detektPlugins", "$RULES_GROUP:detekt:$rulesVersion")
        val configService = gradle.sharedServices.registerIfAbsent(
            CONFIG_SERVICE_NAME,
            DetektConfigService::class.java
        ) { service ->
            service.parameters.content.set(provider { readResource(ARCHITECTURE_CONFIG_FILE_NAME) })
            service.parameters.configFile.fileValue(
                isolated.rootProject.projectDirectory.file("build/detekt/$ARCHITECTURE_CONFIG_FILE_NAME").asFile
            )
        }
        val architectureConfigFile = configService.map(DetektConfigService::configFile)
        val explicitApiArguments = provider { explicitApiArguments() }
        tasks.withType(Detekt::class.java).configureEach { task ->
            task.usesService(configService)
            task.config.from(architectureConfigFile)
            task.freeCompilerArgs.addAll(explicitApiArguments)
        }
    }

    private fun Project.konsistClasspath(rulesVersion: String): NamedDomainObjectProvider<ResolvableConfiguration> {
        val konsist = configurations.dependencyScope(KONSIST_CONFIGURATION) { scope ->
            scope.dependencies.add(dependencies.create("$RULES_GROUP:konsist:$rulesVersion"))
        }
        return configurations.resolvable("${KONSIST_CONFIGURATION}Classpath") { classpath ->
            classpath.extendsFrom(konsist.get())
            classpath.attributes { attributes ->
                attributes.attribute(Usage.USAGE_ATTRIBUTE, objects.named(Usage::class.java, Usage.JAVA_RUNTIME))
                attributes.attribute(
                    Category.CATEGORY_ATTRIBUTE,
                    objects.named(Category::class.java, Category.LIBRARY)
                )
                attributes.attribute(
                    LibraryElements.LIBRARY_ELEMENTS_ATTRIBUTE,
                    objects.named(LibraryElements::class.java, LibraryElements.JAR)
                )
                attributes.attribute(
                    Bundling.BUNDLING_ATTRIBUTE,
                    objects.named(Bundling::class.java, Bundling.EXTERNAL)
                )
                attributes.attribute(
                    TargetJvmEnvironment.TARGET_JVM_ENVIRONMENT_ATTRIBUTE,
                    objects.named(TargetJvmEnvironment::class.java, TargetJvmEnvironment.STANDARD_JVM)
                )
            }
        }
    }

    private fun Project.architectureInputs(): FileCollection {
        return fileTree(projectDir) { tree ->
            tree.include(
                "src/**/*.kt",
                "src/**/*.java",
                "README.md",
                "**/build.gradle.kts",
                "**/build.gradle"
            )
            tree.exclude("**/build/**", "**/.gradle/**")
        }
    }

    private fun ArchitectureTask.configureFrom(
        project: Project,
        extension: LinterExtension,
        classpath: NamedDomainObjectProvider<ResolvableConfiguration>
    ) {
        group = LifecycleBasePlugin.VERIFICATION_GROUP
        konsistClasspath.from(classpath)
        sources.from(project.architectureInputs())
        projectDirectory.set(project.layout.projectDirectory)
        rootDirectory.set(project.isolated.rootProject.projectDirectory)
        projectPath.set(project.path)
        unitKind.set(extension.unitKind)
        rootPackage.set(extension.rootPackage)
        rootOfBuild.set(project.path == project.isolated.rootProject.path)
        projectDependencies.set(project.provider { project.productionProjectDependencies() })
        buildFindings.set(
            extension.unitKind.map { cliName ->
                val kind = UnitKind.fromCliName(cliName) ?: UnitKind.CONTEXT
                project.readUnitBuild(kind).findBuildFindings().map(ArchitectureFinding::toLine)
            }
        )
    }

    private fun Project.graphService(): Provider<ArchitectureGraphService> {
        return gradle.sharedServices.registerIfAbsent(
            GRAPH_SERVICE_NAME,
            ArchitectureGraphService::class.java
        ) { }
    }

    private fun Project.configureGraphContribution() {
        val service = graphService()
        afterEvaluate { project ->
            val parameters = service.get().parameters
            synchronized(parameters) {
                val dependencies = project.productionProjectDependencies()
                val metadata = listOf(project.path).plus(dependencies).joinToString("\t")
                parameters.unitMetadata.add(metadata)
            }
        }
    }

    private fun Project.configureGraphCheck() {
        val service = graphService()
        tasks.register(GRAPH_CHECK_TASK_NAME, ArchitectureGraphTask::class.java) { task ->
            task.description = "Checks that the complete unit dependency graph is acyclic"
            task.usesService(service)
            task.unitMetadata.set(service.get().parameters.unitMetadata.map { metadata -> metadata })
        }
        plugins.withType(LifecycleBasePlugin::class.java) {
            tasks.named(LifecycleBasePlugin.CHECK_TASK_NAME) { task ->
                task.dependsOn(GRAPH_CHECK_TASK_NAME)
            }
        }
    }

    override fun apply(target: Project) {
        target.pluginManager.apply(DetektPlugin::class.java)
        val rulesVersion = readRulesVersion()
        target.configureDetekt(rulesVersion)
        target.configureGraphContribution()

        val extension = target.extensions.create(EXTENSION_NAME, LinterExtension::class.java)
        extension.unitKind.convention(target.provider { target.detectUnitKind().cliName })
        val classpath = target.konsistClasspath(rulesVersion)

        val checkTask = target.tasks.register(CHECK_TASK_NAME, ArchitectureCheckTask::class.java) { task ->
            task.description = "Checks the package layout and the unit structure against the architecture rules"
            task.configureFrom(
                target,
                extension,
                classpath
            )
            task.baseline.from(target.layout.projectDirectory.file(BASELINE_FILE_NAME))
            task.reportFile.set(target.layout.buildDirectory.file("reports/architecture/violations.txt"))
        }
        target.tasks.register(BASELINE_TASK_NAME, ArchitectureBaselineTask::class.java) { task ->
            task.description = "Accepts the current architecture violations in $BASELINE_FILE_NAME"
            task.configureFrom(
                target,
                extension,
                classpath
            )
            task.baselineFile.set(target.layout.projectDirectory.file(BASELINE_FILE_NAME))
        }
        target.plugins.withType(LifecycleBasePlugin::class.java) {
            target.tasks.named(LifecycleBasePlugin.CHECK_TASK_NAME) { task -> task.dependsOn(checkTask) }
        }
        if (target.path == target.isolated.rootProject.path) target.configureGraphCheck()
    }

    companion object {
        const val CHECK_TASK_NAME = "architectureCheck"
        const val GRAPH_CHECK_TASK_NAME = "architectureGraphCheck"
        const val BASELINE_TASK_NAME = "architectureBaseline"
        const val BASELINE_FILE_NAME = "architecture-baseline.txt"

        private const val EXTENSION_NAME = "linter"
        private const val RULES_GROUP = "ru.astrainteractive.klibs.rules"
        private const val RULES_VERSION_FILE = "klibs-rules.properties"
        private const val ARCHITECTURE_CONFIG_FILE_NAME = "detekt-architecture.yml"
        private const val CONFIG_SERVICE_NAME = "klibsDetektArchitectureConfig"
        private const val GRAPH_SERVICE_NAME = "klibsArchitectureGraph"
        private const val KONSIST_CONFIGURATION = "klibsArchitectureKonsist"
    }
}
