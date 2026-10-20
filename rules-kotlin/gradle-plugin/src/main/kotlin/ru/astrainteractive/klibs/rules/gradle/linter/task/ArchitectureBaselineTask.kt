package ru.astrainteractive.klibs.rules.gradle.linter.task

import org.gradle.api.file.RegularFileProperty
import org.gradle.api.tasks.OutputFile
import org.gradle.api.tasks.TaskAction
import org.gradle.work.DisableCachingByDefault

@DisableCachingByDefault(because = "The baseline is a source file of the project, not a build output to cache")
internal abstract class ArchitectureBaselineTask : ArchitectureTask() {
    @get:OutputFile
    abstract val baselineFile: RegularFileProperty

    @TaskAction
    fun writeBaseline() {
        val keys = findFindings().map { finding -> finding.baselineKey }.distinct().sorted()
        val file = baselineFile.get().asFile
        if (keys.isEmpty()) {
            file.delete()
            return
        }
        file.writeText(keys.joinToString(separator = "") { key -> key + "\n" })
    }
}
