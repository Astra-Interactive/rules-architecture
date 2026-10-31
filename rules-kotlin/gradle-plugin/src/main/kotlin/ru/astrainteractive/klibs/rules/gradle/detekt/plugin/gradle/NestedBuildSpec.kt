package ru.astrainteractive.klibs.rules.gradle.detekt.plugin.gradle

import org.gradle.api.file.FileTreeElement
import org.gradle.api.specs.Spec
import java.io.File

/**
 * Matches a directory inside the project directory that is a build or a checkout of its own: a Gradle build or
 * settings file, or a `.git` entry, as a worktree under `.claude/worktrees` has. The project's detekt leaves such a
 * directory to that build's own detekt.
 */
internal class NestedBuildSpec : Spec<FileTreeElement> {
    override fun isSatisfiedBy(element: FileTreeElement): Boolean {
        if (!element.isDirectory) return false
        return BUILD_MARKERS.any { marker -> File(element.file, marker).exists() }
    }

    private companion object {
        @JvmField
        val BUILD_MARKERS = listOf(
            "build.gradle.kts",
            "build.gradle",
            "settings.gradle.kts",
            "settings.gradle",
            ".git"
        )
    }
}
