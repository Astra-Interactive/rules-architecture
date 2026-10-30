package ru.astrainteractive.klibs.rules.gradle.linter.task.gradle

import org.gradle.api.DefaultTask
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.provider.SetProperty
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.OutputFile
import org.gradle.api.tasks.TaskAction
import org.gradle.work.DisableCachingByDefault

/**
 * Writes the unit's technology words into the settings of the detekt rules that judge technology packages, so that
 * detekt and the Konsist checks know the same words.
 */
@DisableCachingByDefault(because = "Writing a few lines is cheaper than caching them")
internal abstract class DetektWordsTask : DefaultTask() {
    @get:Input
    abstract val technologyWords: SetProperty<String>

    @get:OutputFile
    abstract val configFile: RegularFileProperty

    @TaskAction
    fun write() {
        val words = technologyWords.get().sorted().joinToString { word -> "'$word'" }
        val content = buildString {
            appendLine("architecture:")
            RULES.forEach { rule ->
                appendLine("  $rule:")
                appendLine("    technologyWords: [ $words ]")
            }
        }
        val file = configFile.get().asFile
        file.parentFile.mkdirs()
        file.writeText(content)
    }

    private companion object {
        @JvmField
        val RULES = listOf(
            "BareIdentifier",
            "KindNameInPackage",
            "RunBlockingOutsideBoundary"
        )
    }
}
