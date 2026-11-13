// A Gradle plugin is the composition root of its build: it registers the build services it shares.
@file:Suppress("KindImportDirection")

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
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskProvider
import org.gradle.language.base.plugins.LifecycleBasePlugin
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.util.encode
import ru.astrainteractive.klibs.rules.core.unit.util.toUnitContext
import ru.astrainteractive.klibs.rules.gradle.detekt.extension.model.DetektRulesExtension
import ru.astrainteractive.klibs.rules.gradle.detekt.plugin.gradle.DetektPlugin
import ru.astrainteractive.klibs.rules.gradle.detekt.service.DetektConfigService
import ru.astrainteractive.klibs.rules.gradle.linter.extension.model.LinterExtension
import ru.astrainteractive.klibs.rules.gradle.linter.model.AndroidVariantSources
import ru.astrainteractive.klibs.rules.gradle.linter.model.ArchitectureFinding
import ru.astrainteractive.klibs.rules.gradle.linter.service.ArchitectureGraphService
import ru.astrainteractive.klibs.rules.gradle.linter.task.gradle.ApiOwnershipReportTask
import ru.astrainteractive.klibs.rules.gradle.linter.task.gradle.ArchitectureBaselineTask
import ru.astrainteractive.klibs.rules.gradle.linter.task.gradle.ArchitectureCheckTask
import ru.astrainteractive.klibs.rules.gradle.linter.task.gradle.ArchitectureGraphTask
import ru.astrainteractive.klibs.rules.gradle.linter.task.gradle.ArchitectureTask
import ru.astrainteractive.klibs.rules.gradle.linter.task.gradle.DetektSettingsTask
import ru.astrainteractive.klibs.rules.gradle.linter.util.androidNamespace
import ru.astrainteractive.klibs.rules.gradle.linter.util.dependencyInputs
import ru.astrainteractive.klibs.rules.gradle.linter.util.explicitApiMode
import ru.astrainteractive.klibs.rules.gradle.linter.util.findBuildFindings
import ru.astrainteractive.klibs.rules.gradle.linter.util.findUnitTechnologyWords
import ru.astrainteractive.klibs.rules.gradle.linter.util.hasMikroCoreInCatalog
import ru.astrainteractive.klibs.rules.gradle.linter.util.productionProjectDependencies
import ru.astrainteractive.klibs.rules.gradle.linter.util.readUnitBuild
import ru.astrainteractive.klibs.rules.gradle.linter.util.readUnitContext
import ru.astrainteractive.klibs.rules.gradle.linter.util.registerAndroidVariantSources
import ru.astrainteractive.klibs.rules.gradle.linter.util.toLine
import ru.astrainteractive.klibs.rules.gradle.util.hasKotlinPlugin
import java.io.File
import java.util.Properties

@Suppress("TooManyFunctions") // The composition root owns each task and lifecycle registration.
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

    private fun Project.graphParameters(): ArchitectureGraphService.Parameters {
        gradle.sharedServices.registerIfAbsent(
            GRAPH_SERVICE_NAME,
            ArchitectureGraphService::class.java
        ) { }
        val registration = requireNotNull(gradle.sharedServices.registrations.findByName(GRAPH_SERVICE_NAME))
        return registration.parameters as ArchitectureGraphService.Parameters
    }

    private fun Project.configureDetekt(rulesVersion: String, context: Provider<UnitContext>) {
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
        val settingsTask = tasks.register(
            DETEKT_SETTINGS_TASK_NAME,
            DetektSettingsTask::class.java
        ) { task ->
            task.technologyWords.set(context.map(UnitContext::technologyWords))
            task.unitContext.set(context.map(UnitContext::encode))
            task.inlineCancellationCheck.set(provider { !hasMikroCoreInCatalog() })
            task.configFile.set(layout.buildDirectory.file("detekt/$DETEKT_SETTINGS_FILE_NAME"))
            task.unitContextFile.set(layout.buildDirectory.file("detekt/unit-context.txt"))
        }
        val settingsFile = settingsTask.flatMap(DetektSettingsTask::configFile)
        extensions.getByType(DetektRulesExtension::class.java).ruleConfigs.from(architectureConfigFile, settingsFile)
        val explicitApiArguments = provider { explicitApiArguments() }
        tasks.withType(Detekt::class.java).configureEach { task ->
            task.usesService(configService)
            task.inputs.file(settingsTask.flatMap(DetektSettingsTask::unitContextFile))
                .withPropertyName("architectureUnitContext")
                .withPathSensitivity(PathSensitivity.NONE)
            // Detekt creates separate tasks for compilations and KMP source sets. Public is required
            // in their production sources when Kotlin explicit API mode is enabled. Match the Test
            // token, not its letters inside a production name such as LatestMain.
            if (!TEST_TASK_TOKEN.containsMatchIn(task.name)) {
                task.freeCompilerArgs.addAll(explicitApiArguments)
            }
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
        context: Provider<UnitContext>,
        classpath: NamedDomainObjectProvider<ResolvableConfiguration>
    ) {
        group = LifecycleBasePlugin.VERIFICATION_GROUP
        konsistClasspath.from(classpath)
        sources.from(project.architectureInputs())
        dependencySources.from(project.provider { project.dependencyInputs() })
        projectDirectory.set(project.layout.projectDirectory)
        rootDirectory.set(project.isolated.rootProject.projectDirectory)
        projectPath.set(project.path)
        unitContext.set(context.map(UnitContext::encode))
        unitMetadata.set(project.graphParameters().unitMetadata.map { metadata -> metadata })
        rootPackage.set(extension.rootPackage)
        androidNamespace.set(project.provider { project.androidNamespace() })
        entryPointWords.set(extension.entryPointWords)
        technologyWords.set(project.findUnitTechnologyWords(extension))
        pluralConcepts.set(extension.pluralConcepts)
        platformClasses.set(extension.platformClasses)
        extensionKinds.set(extension.extensionKinds)
        roleKinds.set(extension.roleKinds)
        rootOfBuild.set(project.path == project.isolated.rootProject.path)
        conventionPlugin.set(
            project.provider {
                val kotlinDsl = project.pluginManager.hasPlugin("org.gradle.kotlin.kotlin-dsl")
                val groovyDsl = project.pluginManager.hasPlugin("groovy-gradle-plugin")
                kotlinDsl || groovyDsl
            }
        )
        projectDependencies.set(project.provider { project.productionProjectDependencies() })
    }

    private fun Project.configureGraphContribution(
        extension: LinterExtension,
        androidVariants: List<AndroidVariantSources>
    ): Provider<UnitContext> {
        val rootProduct = graphParameters().productKind
        val context = provider {
            readUnitContext(
                extension,
                rootProduct.orNull,
                androidVariants = androidVariants
            )
        }
        afterEvaluate { project ->
            val parameters = project.graphParameters()
            if (project.path == project.isolated.rootProject.path) {
                extension.configuredProductKind?.let(parameters.productKind::set)
            }
        }
        gradle.projectsEvaluated {
            val parameters = graphParameters()
            // Root graph inputs must not resolve another project's compilation classpath.
            val snapshot = readUnitContext(
                extension,
                parameters.productKind.orNull,
                resolveCompose = false,
                androidVariants = androidVariants
            )
            val metadata = listOf(path, snapshot.encode())
                .plus(productionProjectDependencies())
                .joinToString("\t")
            synchronized(parameters) {
                parameters.unitMetadata.add(metadata)
                parameters.ownershipMetadata.add(
                    listOf(
                        path,
                        projectDir.absolutePath,
                        snapshot.encode()
                    ).joinToString("\t")
                )
            }
        }
        return context
    }

    private fun Project.configureGraphReports(classpath: NamedDomainObjectProvider<ResolvableConfiguration>) {
        tasks.register(
            GRAPH_CHECK_TASK_NAME,
            ArchitectureGraphTask::class.java
        ) { task ->
            task.description = "Checks that the unit graph is acyclic and that nothing depends on an entry point"
            task.unitMetadata.set(graphParameters().unitMetadata.map { metadata -> metadata })
        }
        plugins.withType(LifecycleBasePlugin::class.java) {
            tasks.named(LifecycleBasePlugin.CHECK_TASK_NAME) { task ->
                task.dependsOn(GRAPH_CHECK_TASK_NAME)
            }
        }
        tasks.register(
            OWNERSHIP_REPORT_TASK_NAME,
            ApiOwnershipReportTask::class.java
        ) { task ->
            task.group = LifecycleBasePlugin.VERIFICATION_GROUP
            task.description = "Reports API declarations with a sole sibling implementation consumer for manual review"
            task.configurationOnDemand.set(gradle.startParameter.isConfigureOnDemand)
            if (gradle.startParameter.isConfigureOnDemand) {
                task.notCompatibleWithConfigurationCache("API ownership reports require all projects configured")
            }
            val metadata = graphParameters().ownershipMetadata.map { entries -> entries.sorted() }
            task.unitMetadata.set(metadata)
            task.konsistClasspath.from(classpath)
            task.sources.from(
                metadata.map { entries ->
                    entries.flatMap { entry ->
                        entry.split('\t')[2].toUnitContext().compilations.filterNot(UnitContext.Compilation::test)
                            .flatMap { compilation -> compilation.sourceSets.values.flatten() }
                    }.distinct().map(::File).filter(File::isDirectory)
                }
            )
            task.reportFile.set(layout.buildDirectory.file("reports/architecture/api-ownership.txt"))
        }
    }

    private fun Project.attachCheckTask(checkTask: TaskProvider<ArchitectureCheckTask>) {
        plugins.withType(LifecycleBasePlugin::class.java) {
            tasks.named(LifecycleBasePlugin.CHECK_TASK_NAME) { task -> task.dependsOn(checkTask) }
        }
    }

    override fun apply(target: Project) {
        target.pluginManager.apply(DetektPlugin::class.java)
        val androidVariants = target.registerAndroidVariantSources()
        val rulesVersion = readRulesVersion()
        val extension = target.extensions.create(EXTENSION_NAME, LinterExtension::class.java)
        val context = target.configureGraphContribution(extension, androidVariants)
        target.configureDetekt(rulesVersion, context)
        val classpath = target.konsistClasspath(rulesVersion)

        val checkTask = target.tasks.register(
            CHECK_TASK_NAME,
            ArchitectureCheckTask::class.java
        ) { task ->
            task.description = "Checks the package layout and the unit structure against the architecture rules"
            task.configureFrom(
                target,
                extension,
                context,
                classpath
            )
            task.baseline.from(target.layout.projectDirectory.file(BASELINE_FILE_NAME))
            task.baselineTaskPath.set("${target.path.removeSuffix(":")}:$BASELINE_TASK_NAME")
            task.reportFile.set(target.layout.buildDirectory.file("reports/architecture/violations.txt"))
        }
        val baselineTask = target.tasks.register(
            BASELINE_TASK_NAME,
            ArchitectureBaselineTask::class.java
        ) { task ->
            task.description = "Accepts the current architecture violations in $BASELINE_FILE_NAME"
            task.configureFrom(
                target,
                extension,
                context,
                classpath
            )
            task.baselineFile.set(target.layout.projectDirectory.file(BASELINE_FILE_NAME))
        }
        target.afterEvaluate { project ->
            val unit = project.readUnitContext(
                extension,
                project.graphParameters().productKind.orNull,
                resolveCompose = false,
                androidVariants = androidVariants
            )
            val findings = project.readUnitBuild(unit.kind, unit.productKind)
                .findBuildFindings().map(ArchitectureFinding::toLine)
            checkTask.configure { task -> task.buildFindings.set(findings) }
            baselineTask.configure { task -> task.buildFindings.set(findings) }
        }
        target.attachCheckTask(checkTask)
        val lintTask = target.tasks.register(LINT_TASK_NAME) { task ->
            task.group = LifecycleBasePlugin.VERIFICATION_GROUP
            task.description = "Runs every check of the linter, detekt with type resolution included, but no tests"
            task.dependsOn(checkTask)
            task.dependsOn(target.tasks.withType(Detekt::class.java).named { name -> name != DETEKT_FORMAT_TASK_NAME })
        }
        if (target.path == target.isolated.rootProject.path) {
            target.configureGraphReports(classpath)
            lintTask.configure { task -> task.dependsOn(GRAPH_CHECK_TASK_NAME, OWNERSHIP_REPORT_TASK_NAME) }
        }
    }

    companion object {
        const val CHECK_TASK_NAME = "architectureCheck"
        const val LINT_TASK_NAME = "architectureLint"
        private const val DETEKT_FORMAT_TASK_NAME = "detektFormat"
        private const val DETEKT_TASK_NAME = "detekt"
        const val GRAPH_CHECK_TASK_NAME = "architectureGraphCheck"
        const val OWNERSHIP_REPORT_TASK_NAME = "architectureApiOwnershipReport"
        const val BASELINE_TASK_NAME = "architectureBaseline"
        const val BASELINE_FILE_NAME = "architecture-baseline.txt"
        const val DETEKT_SETTINGS_TASK_NAME = "architectureDetektSettings"
        private const val DETEKT_SETTINGS_FILE_NAME = "architecture-settings.yml"

        private const val EXTENSION_NAME = "linter"
        private const val RULES_GROUP = "ru.astrainteractive.klibs.rules"

        // Namespaced like the bundled detekt.yml, so another plugin of the build cannot shadow it
        private const val RULES_VERSION_FILE = "ru/astrainteractive/klibs/rules/klibs-rules.properties"
        private const val ARCHITECTURE_CONFIG_FILE_NAME = "detekt-architecture.yml"
        private const val CONFIG_SERVICE_NAME = "klibsDetektArchitectureConfig"
        private const val GRAPH_SERVICE_NAME = "klibsArchitectureGraph"
        private const val KONSIST_CONFIGURATION = "klibsArchitectureKonsist"
        private val TEST_TASK_TOKEN = Regex("Tests?(?=[A-Z]|$)")
    }
}
