package ru.astrainteractive.klibs.rules.gradle.linter.util

import org.gradle.api.Project
import org.jetbrains.kotlin.gradle.dsl.ExplicitApiMode
import org.jetbrains.kotlin.gradle.dsl.KotlinBaseExtension

/**
 * Call it only when a Kotlin plugin is applied.
 */
internal fun Project.explicitApiMode(): String? {
    return when (extensions.findByType(KotlinBaseExtension::class.java)?.explicitApi) {
        ExplicitApiMode.Strict -> "strict"
        ExplicitApiMode.Warning -> "warning"
        else -> null
    }
}
