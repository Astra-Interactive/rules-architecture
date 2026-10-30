package ru.astrainteractive.klibs.rules.gradle.linter.task.gradle

import org.gradle.api.DefaultTask
import org.gradle.api.GradleException
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
import ru.astrainteractive.klibs.rules.gradle.linter.model.ArchitectureFinding
import ru.astrainteractive.klibs.rules.gradle.linter.model.UnitKind
import ru.astrainteractive.klibs.rules.structure.buildfile.structure.RootBuildSubprojects
import ru.astrainteractive.klibs.rules.structure.graph.structure.ApiUnitDependency
import ru.astrainteractive.klibs.rules.structure.graph.structure.EntryPointDependency
import ru.astrainteractive.klibs.rules.structure.graph.structure.TechnologyDependency
import ru.astrainteractive.klibs.rules.structure.graph.structure.UnitDependency
import ru.astrainteractive.klibs.rules.structure.nesting.structure.NestedUnit
import ru.astrainteractive.klibs.rules.structure.nesting.structure.ProjectDirMatchesPath
import ru.astrainteractive.klibs.rules.structure.nesting.structure.RootSources
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

    @get:Internal
    abstract val projectDirectory: DirectoryProperty

    @get:Internal
    abstract val rootDirectory: DirectoryProperty

    @get:Input
    abstract val projectPath: Property<String>

    @get:Input
    abstract val unitKind: Property<String>

    @get:Input
    @get:Optional
    abstract val rootPackage: Property<String>

    @get:Input
    abstract val rootOfBuild: Property<Boolean>

    @get:Input
    abstract val entryPointWords: SetProperty<String>

    @get:Input
    abstract val technologyWords: SetProperty<String>

    @get:Input
    abstract val pluralConcepts: SetProperty<String>

    @get:Input
    abstract val projectDependencies: SetProperty<String>

    /**
     * The findings of the unit's own build, read at configuration time, one report line each.
     */
    @get:Input
    abstract val buildFindings: ListProperty<String>

    @get:Inject
    abstract val execOperations: ExecOperations

    private fun sourceDirectories(isTest: Boolean): List<File> {
        return File(projectDirectory.get().asFile, SOURCE_ROOT).listFiles().orEmpty()
            .filter(File::isDirectory)
            .filter { sourceSet -> sourceSet.name.contains(TEST_MARKER, ignoreCase = true) == isTest }
            .flatMap { sourceSet -> LANGUAGE_DIRECTORIES.map { language -> File(sourceSet, language) } }
            .filter(File::isDirectory)
            .sortedBy(File::getPath)
    }

    private fun wordArguments(): List<String> {
        val words = mapOf(
            "--entry-point-word" to entryPointWords.get(),
            "--technology-word" to technologyWords.get(),
            "--plural-concept" to pluralConcepts.get()
        )
        return words.flatMap { (option, values) -> values.sorted().flatMap { value -> listOf(option, value) } }
    }

    private fun konsistArguments(
        kind: UnitKind,
        mainDirectories: List<File>,
        reportFile: File
    ): List<String> {
        return buildList {
            add("--project-dir")
            add(projectDirectory.get().asFile.absolutePath)
            add("--unit-kind")
            add(kind.cliName)
            add("--report")
            add(reportFile.absolutePath)
            add("--unit-path")
            add(projectPath.get())
            rootPackage.orNull?.let { packageName ->
                add("--root-package")
                add(packageName)
            }
            mainDirectories.forEach { directory ->
                add("--main")
                add(directory.parentFile.name + "=" + directory.absolutePath)
            }
            sourceDirectories(isTest = true).forEach { directory ->
                add("--test")
                add(directory.parentFile.name + "=" + directory.absolutePath)
            }
            addAll(wordArguments())
        }
    }

    private fun runKonsist(kind: UnitKind): List<ArchitectureFinding> {
        val mainDirectories = sourceDirectories(isTest = false)
        if (rootOfBuild.get() || mainDirectories.isEmpty()) return emptyList()
        val reportFile = File(temporaryDir, KONSIST_REPORT)
        // Konsist embeds the Kotlin compiler, so it runs in a JVM of its own
        execOperations.javaexec { spec ->
            spec.classpath = konsistClasspath
            spec.mainClass.set(KONSIST_MAIN_CLASS)
            val arguments = konsistArguments(
                kind,
                mainDirectories,
                reportFile
            )
            spec.args(arguments)
        }
        return reportFile.readLines()
            .filter(String::isNotBlank)
            .map(ArchitectureFinding::parse)
    }

    private fun runStructureChecks(kind: UnitKind): List<ArchitectureFinding> {
        val unit = BuildUnit(
            path = projectPath.get(),
            directory = projectDirectory.get().asFile,
            isEntryPoint = kind == UnitKind.ENTRY_POINT,
            isRoot = rootOfBuild.get(),
            projectDependencies = projectDependencies.get(),
            isLibrary = kind == UnitKind.LIBRARY
        )
        val checker = StructureChecker(
            listOf(
                ApiUnitDependency(),
                EntryPointDependency(),
                NestedUnit(),
                ProjectDirMatchesPath(rootDirectory.get().asFile),
                RootBuildSubprojects(),
                RootSources(),
                TechnologyDependency(),
                UnitDependency(),
                UnitReadme()
            )
        )
        return checker.check(unit).map { violation ->
            ArchitectureFinding(
                violation.rule,
                violation.path,
                violation.subject,
                violation.message
            )
        }
    }

    protected fun findFindings(): List<ArchitectureFinding> {
        val kind = UnitKind.fromCliName(unitKind.get()) ?: throw GradleException(
            "linter.unitKind of ${projectPath.get()} is '${unitKind.get()}', expected one of " +
                UnitKind.entries.joinToString(transform = UnitKind::cliName)
        )
        val buildFindings = buildFindings.get().map(ArchitectureFinding::parse)
        return runKonsist(kind) + runStructureChecks(kind) + buildFindings
    }

    private companion object {
        const val SOURCE_ROOT = "src"
        const val TEST_MARKER = "test"
        const val KONSIST_REPORT = "konsist.tsv"
        const val KONSIST_MAIN_CLASS = "ru.astrainteractive.klibs.rules.konsist.cli.konsist.KonsistCli"

        val LANGUAGE_DIRECTORIES = listOf("kotlin", "java")
    }
}
