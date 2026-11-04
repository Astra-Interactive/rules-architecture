package ru.astrainteractive.klibs.rules.gradle.linter.task.gradle

import org.gradle.api.DefaultTask
import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.provider.ListProperty
import org.gradle.api.tasks.Classpath
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.InputFiles
import org.gradle.api.tasks.OutputFile
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction
import org.gradle.process.ExecOperations
import org.gradle.work.DisableCachingByDefault
import java.io.File
import javax.inject.Inject

@DisableCachingByDefault(because = "The advisory report includes absolute source roots for coverage review")
internal abstract class ApiOwnershipReportTask : DefaultTask() {
    @get:Input
    abstract val unitMetadata: ListProperty<String>

    @get:Classpath
    abstract val konsistClasspath: ConfigurableFileCollection

    @get:InputFiles
    @get:PathSensitive(PathSensitivity.RELATIVE)
    abstract val sources: ConfigurableFileCollection

    @get:OutputFile
    abstract val reportFile: RegularFileProperty

    @get:Inject
    abstract val execOperations: ExecOperations

    @TaskAction
    fun writeReport() {
        val metadata = File(temporaryDir, "ownership-units.tsv")
        metadata.writeText(unitMetadata.get().sorted().joinToString("\n"))
        execOperations.javaexec { spec ->
            spec.classpath = konsistClasspath
            spec.mainClass.set("ru.astrainteractive.klibs.rules.konsist.ownership.konsist.ApiOwnershipCli")
            spec.args(metadata.absolutePath, reportFile.get().asFile.absolutePath)
        }
        logger.lifecycle("API ownership review: ${reportFile.get().asFile}")
    }
}
