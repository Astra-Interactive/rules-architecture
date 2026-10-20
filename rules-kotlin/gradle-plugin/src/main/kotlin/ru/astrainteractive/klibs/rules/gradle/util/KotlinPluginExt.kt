package ru.astrainteractive.klibs.rules.gradle.util

import org.gradle.api.Project

private val KOTLIN_PLUGIN_IDS = listOf(
    "org.jetbrains.kotlin.jvm",
    "org.jetbrains.kotlin.multiplatform",
    "org.jetbrains.kotlin.android",
)

/**
 * Tells whether a Kotlin Gradle plugin is applied, so the code that loads Kotlin Gradle plugin types, which a
 * build without Kotlin does not have, is called only when it is.
 */
internal fun Project.hasKotlinPlugin(): Boolean {
    return KOTLIN_PLUGIN_IDS.any { pluginId -> pluginManager.hasPlugin(pluginId) }
}
