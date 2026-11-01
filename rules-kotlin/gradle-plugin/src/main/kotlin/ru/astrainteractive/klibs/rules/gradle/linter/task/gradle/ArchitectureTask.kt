package ru.astrainteractive.klibs.rules.gradle.linter.task.gradle

import org.gradle.api.DefaultTask
import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.file.DirectoryProperty
import org.gradle.api.provider.ListProperty
import org.gradle.api.provider.Property
import org.gradle.api.provider.SetProperty
import org.gradle.api.tasks.Classpath
import org.gradle.api.tasks.IgnoreEmptyDirectories
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.InputFiles
import org.gradle.api.tasks.Internal
import org.gradle.api.tasks.Optional
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.process.ExecOperations
import org.gradle.work.DisableCachingByDefault
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.core.unit.util.encode
import ru.astrainteractive.klibs.rules.core.unit.util.toUnitContext
import ru.astrainteractive.klibs.rules.gradle.linter.model.ArchitectureFinding
import ru.astrainteractive.klibs.rules.structure.buildfile.structure.RootBuildSubprojects
import ru.astrainteractive.klibs.rules.structure.graph.structure.ApiUnitDependency
import ru.astrainteractive.klibs.rules.structure.graph.structure.EntryPointDependency
import ru.astrainteractive.klibs.rules.structure.graph.structure.PluginImplUnit
import ru.astrainteractive.klibs.rules.structure.graph.structure.TechnologyDependency
import ru.astrainteractive.klibs.rules.structure.graph.structure.UnitDependency
import ru.astrainteractive.klibs.rules.structure.nesting.structure.ContainerSources
import ru.astrainteractive.klibs.rules.structure.nesting.structure.NestedUnit
import ru.astrainteractive.klibs.rules.structure.nesting.structure.PlatformSourceSetUnit
import ru.astrainteractive.klibs.rules.structure.nesting.structure.ProjectDirMatchesPath
import ru.astrainteractive.klibs.rules.structure.nesting.structure.RootSources
import ru.astrainteractive.klibs.rules.structure.nesting.structure.UnitPath
import ru.astrainteractive.klibs.rules.structure.readme.structure.UnitReadme
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.structure.StructureChecker
import java.io.File
import javax.inject.Inject

@DisableCachingByDefault(because = "An abstract base: each concrete task declares its own caching")
internal abstract class ArchitectureTask : DefaultTask() {
    @get:Classpath
    abstract val konsistClasspath: ConfigurableFileCollection

    @get:InputFiles
    @get:PathSensitive(PathSensitivity.RELATIVE)
    @get:IgnoreEmptyDirectories
    abstract val sources: ConfigurableFileCollection

    /**
     * The sources of the units this one depends on, declared so that a change in a dependency's model reruns the
     * check.
     */
    @get:InputFiles
    @get:PathSensitive(PathSensitivity.RELATIVE)
    @get:IgnoreEmptyDirectories
    abstract val dependencySources: ConfigurableFileCollection

    @get:Internal
    abstract val projectDirectory: DirectoryProperty

    @get:Internal
    abstract val rootDirectory: DirectoryProperty

    @get:Input
    abstract val projectPath: Property<String>

    @get:Input
    abstract val unitContext: Property<String>

    @get:Input
    abstract val unitMetadata: ListProperty<String>

    @get:Input
    @get:Optional
    abstract val rootPackage: Property<String>

    @get:Input
    abstract val rootOfBuild: Property<Boolean>

    @get:Input
    @get:Optional
    abstract val androidNamespace: Property<String>

    @get:Input
    abstract val entryPointWords: SetProperty<String>

    @get:Input
    abstract val technologyWords: SetProperty<String>

    @get:Input
    abstract val pluralConcepts: SetProperty<String>

    @get:Input
    abstract val platformClasses: SetProperty<String>

    @get:Input
    abstract val extensionKinds: SetProperty<String>

    @get:Input
    abstract val projectDependencies: SetProperty<String>

    /**
     * The findings of the unit's own build, read at configuration time, one report line each.
     */
    @get:Input
    abstract val buildFindings: ListProperty<String>

    @get:Inject
    abstract val execOperations: ExecOperations

    private fun sourceDirectories(unitDirectory: File, isTest: Boolean): List<File> {
        return File(unitDirectory, SOURCE_ROOT).listFiles().orEmpty()
            .filter(File::isDirectory)
            .filter { sourceSet -> sourceSet.name.contains(TEST_MARKER, ignoreCase = true) == isTest }
            .flatMap { sourceSet -> LANGUAGE_DIRECTORIES.map { language -> File(sourceSet, language) } }
            .filter(File::isDirectory)
            .sortedBy(File::getPath)
    }

    private fun sourceDirectories(context: UnitContext, isTest: Boolean): List<File> {
        if (context.compilations.isEmpty()) return sourceDirectories(projectDirectory.get().asFile, isTest)
        val main = context.compilations.filterNot(UnitContext.Compilation::test)
            .flatMap { compilation -> compilation.sourceSets.values.flatten() }.toSet()
        val selected = if (isTest) {
            context.compilations.filter(UnitContext.Compilation::test)
                .flatMap { compilation -> compilation.sourceSets.values.flatten() }.toSet() - main
        } else {
            main
        }
        return selected.map(::File).filter(File::isDirectory).sortedBy(File::getPath)
    }

    private fun sourceSetName(context: UnitContext, directory: File): String {
        return context.compilations.asSequence()
            .flatMap { compilation -> compilation.sourceSets.entries.asSequence() }
            .firstOrNull { entry -> directory.absolutePath in entry.value }?.key ?: directory.parentFile.name
    }

    private fun dependencyArguments(): List<String> {
        val root = rootDirectory.get().asFile
        return projectDependencies.get()
            .sorted()
            .map { path -> File(root, path.removePrefix(":").replace(':', '/')) }
            .flatMap { unitDirectory -> sourceDirectories(unitDirectory, isTest = false) }
            .flatMap { directory -> listOf(DEPENDENCY_OPTION, directory.absolutePath) }
    }

    private fun wordArguments(): List<String> {
        val words = mapOf(
            "--entry-point-word" to entryPointWords.get(),
            "--technology-word" to technologyWords.get(),
            "--plural-concept" to pluralConcepts.get(),
            "--platform-class" to platformClasses.get(),
            "--extension-kind" to extensionKinds.get()
        )
        return words.flatMap { (option, values) -> values.sorted().flatMap { value -> listOf(option, value) } }
    }

    private fun konsistArguments(
        context: UnitContext,
        mainDirectories: List<File>,
        reportFile: File
    ): List<String> {
        return buildList {
            add("--project-dir")
            add(projectDirectory.get().asFile.absolutePath)
            add("--unit-kind")
            add(context.kind.cliName)
            add("--unit-context")
            add(context.encode())
            add("--report")
            add(reportFile.absolutePath)
            add("--unit-path")
            add(projectPath.get())
            rootPackage.orNull?.let { packageName ->
                add("--root-package")
                add(packageName)
            }
            androidNamespace.orNull?.let { namespace ->
                add("--android-namespace")
                add(namespace)
            }
            mainDirectories.forEach { directory ->
                add("--main")
                add(sourceSetName(context, directory) + "=" + directory.absolutePath)
            }
            sourceDirectories(context, isTest = true).forEach { directory ->
                add("--test")
                add(sourceSetName(context, directory) + "=" + directory.absolutePath)
            }
            addAll(wordArguments())
            addAll(dependencyArguments())
        }
    }

    private fun runKonsist(context: UnitContext): List<ArchitectureFinding> {
        val mainDirectories = sourceDirectories(context, isTest = false)
        if (rootOfBuild.get() || mainDirectories.isEmpty()) return emptyList()
        val reportFile = File(temporaryDir, KONSIST_REPORT)
        // Konsist embeds the Kotlin compiler, so it runs in a JVM of its own
        execOperations.javaexec { spec ->
            spec.classpath = konsistClasspath
            spec.mainClass.set(KONSIST_MAIN_CLASS)
            val arguments = konsistArguments(
                context,
                mainDirectories,
                reportFile
            )
            spec.args(arguments)
        }
        return reportFile.readLines()
            .filter(String::isNotBlank)
            .map(ArchitectureFinding::parse)
    }

    private fun runStructureChecks(context: UnitContext): List<ArchitectureFinding> {
        val contexts = buildMap {
            unitMetadata.get().forEach { metadata ->
                val parts = metadata.split("\t")
                put(parts[0], parts[1].toUnitContext())
            }
        }
        val unit = BuildUnit(
            path = projectPath.get(),
            directory = projectDirectory.get().asFile,
            isEntryPoint = context.kind == UnitKind.ENTRY_POINT,
            isRoot = rootOfBuild.get(),
            projectDependencies = projectDependencies.get(),
            isLibrary = context.publishedLibrary,
            context = context,
            dependencyContexts = contexts
        )
        val checker = StructureChecker(
            listOf(
                ApiUnitDependency(),
                ContainerSources(rootDirectory.get().asFile),
                EntryPointDependency(),
                NestedUnit(),
                PlatformSourceSetUnit(),
                ProjectDirMatchesPath(rootDirectory.get().asFile),
                RootBuildSubprojects(),
                RootSources(),
                TechnologyDependency(),
                UnitDependency(),
                UnitPath(),
                UnitReadme()
            )
        )
        return checker.check(unit).plus(PluginImplUnit().check(listOf(unit))).map { violation ->
            ArchitectureFinding(
                violation.rule,
                violation.path,
                violation.subject,
                violation.message
            )
        }
    }

    protected fun findFindings(): List<ArchitectureFinding> {
        val context = unitContext.get().toUnitContext()
        val buildFindings = buildFindings.get().map(ArchitectureFinding::parse)
        return runKonsist(context) + runStructureChecks(context) + buildFindings
    }

    private companion object {
        const val SOURCE_ROOT = "src"
        const val DEPENDENCY_OPTION = "--dependency"
        const val TEST_MARKER = "test"
        const val KONSIST_REPORT = "konsist.tsv"
        const val KONSIST_MAIN_CLASS = "ru.astrainteractive.klibs.rules.konsist.cli.konsist.KonsistCli"

        val LANGUAGE_DIRECTORIES = listOf("kotlin", "java")
    }
}
