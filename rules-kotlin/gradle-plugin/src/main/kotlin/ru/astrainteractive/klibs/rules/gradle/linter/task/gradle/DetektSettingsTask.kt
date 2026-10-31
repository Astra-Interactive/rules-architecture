package ru.astrainteractive.klibs.rules.gradle.linter.task.gradle

import org.gradle.api.DefaultTask
import org.gradle.api.file.RegularFileProperty
import org.gradle.api.provider.Property
import org.gradle.api.provider.SetProperty
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.OutputFile
import org.gradle.api.tasks.TaskAction
import org.gradle.work.DisableCachingByDefault

/**
 * Writes the unit's own settings of the architecture detekt rules: its technology words, so that detekt and the
 * Konsist checks know the same words, and whether the inline `CancellationException` check is allowed, which it is
 * only when mikro-core is not in the project's version catalog.
 */
@DisableCachingByDefault(because = "Writing a few lines is cheaper than caching them")
internal abstract class DetektSettingsTask : DefaultTask() {
    @get:Input
    abstract val technologyWords: SetProperty<String>

    @get:Input
    abstract val inlineCancellationCheck: Property<Boolean>

    @get:OutputFile
    abstract val configFile: RegularFileProperty

    @TaskAction
    fun write() {
        val words = technologyWords.get().sorted().joinToString { word -> "'$word'" }
        val content = buildString {
            appendLine("architecture:")
            TECHNOLOGY_RULES.forEach { rule ->
                appendLine("  $rule:")
                appendLine("    technologyWords: [ $words ]")
            }
            appendLine("  RunCatchingCancellation:")
            appendLine("    inlineCancellationCheck: ${inlineCancellationCheck.get()}")
        }
        val file = configFile.get().asFile
        file.parentFile.mkdirs()
        file.writeText(content)
    }

    private companion object {
        @JvmField
        val TECHNOLOGY_RULES = listOf(
            "BareIdentifier",
            "KindNameInPackage",
            "RunBlockingOutsideBoundary"
        )
    }
}
