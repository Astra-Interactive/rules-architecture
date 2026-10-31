package ru.astrainteractive.klibs.rules.gradle.detekt.plugin.gradle

import org.gradle.api.file.FileTreeElement
import org.gradle.api.specs.Spec
import java.io.File

/**
 * Matches what a project's detekt leaves alone: a directory inside the project directory that is a build or a
 * checkout of its own (a Gradle build or settings file, or a `.git` entry, as a worktree under `.claude/worktrees`
 * has), and a file in the `build` directory of a Gradle project, such as the sources moko-resources generates into a
 * source set.
 */
internal object NestedBuildSpec : Spec<FileTreeElement> {
    private const val BUILD_DIRECTORY = "build"

    private val BUILD_FILES = listOf(
        "build.gradle.kts",
        "build.gradle"
    )

    private val BUILD_MARKERS = BUILD_FILES + listOf(
        "settings.gradle.kts",
        "settings.gradle",
        ".git"
    )

    private fun isBuildOutput(directory: File): Boolean {
        val project = directory.parentFile ?: return false
        return directory.name == BUILD_DIRECTORY && BUILD_FILES.any { file -> File(project, file).exists() }
    }

    override fun isSatisfiedBy(element: FileTreeElement): Boolean {
        if (generateSequence(element.file.parentFile, File::getParentFile).any(::isBuildOutput)) return true
        if (!element.isDirectory) return false
        return BUILD_MARKERS.any { marker -> File(element.file, marker).exists() }
    }
}
