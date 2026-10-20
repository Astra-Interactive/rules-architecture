package ru.astrainteractive.klibs.rules.gradle.util

import org.gradle.api.Project

private val KOTLIN_PLUGIN_IDS = listOf(
    "org.jetbrains.kotlin.jvm",
    "org.jetbrains.kotlin.multiplatform",
    "org.jetbrains.kotlin.android",
)

internal fun Project.hasKotlinPlugin(): Boolean {
    return KOTLIN_PLUGIN_IDS.any { pluginId -> pluginManager.hasPlugin(pluginId) }
}
