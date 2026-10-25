package ru.astrainteractive.klibs.rules.gradle.linter.task.gradle

import org.gradle.api.GradleException
import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.tasks.InputFiles
import org.gradle.api.tasks.OutputFile
import org.gradle.api.tasks.PathSensitive
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.TaskAction
import org.gradle.work.DisableCachingByDefault
import ru.astrainteractive.klibs.rules.gradle.linter.model.ArchitectureFinding
import ru.astrainteractive.klibs.rules.gradle.linter.plugin.gradle.LinterPlugin
import ru.astrainteractive.klibs.rules.gradle.linter.util.describe
import java.io.File

@DisableCachingByDefault(because = "The check is cheap next to the cost of storing a failing result")
internal abstract class ArchitectureCheckTask : ArchitectureTask() {
    @get:InputFiles
    @get:PathSensitive(PathSensitivity.NONE)
    abstract val baseline: ConfigurableFileCollection

    @get:OutputFile
    abstract val reportFile: RegularFileProperty

    private fun readBaseline(): Set<String> {
        return baseline.files
            .filter(File::isFile)
            .flatMap(File::readLines)
            .filter(String::isNotBlank)
            .toSet()
    }

    private fun failureMessage(findings: List<ArchitectureFinding>): String {
        return buildString {
            appendLine("Architecture check found ${findings.size} violation(s) in ${projectPath.get()}:")
            findings.forEach { finding -> appendLine("  ${finding.describe()}") }
            val baselineTask = "${projectPath.get().removeSuffix(":")}:${LinterPlugin.BASELINE_TASK_NAME}"
            append("Fix them, or run $baselineTask to accept the current ones.")
        }
    }

    @TaskAction
    fun check() {
        val accepted = readBaseline()
        val findings = findFindings().filterNot { finding -> finding.baselineKey in accepted }
        val report = reportFile.get().asFile
        report.parentFile.mkdirs()
        report.writeText(findings.joinToString(separator = "") { finding -> finding.describe() + "\n" })
        if (findings.isEmpty()) return
        throw GradleException(failureMessage(findings))
    }
}
