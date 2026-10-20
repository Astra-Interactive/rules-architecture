package ru.astrainteractive.klibs.rules.gradle.detekt.util

import org.gradle.api.Project
import org.jetbrains.kotlin.gradle.dsl.KotlinJvmProjectExtension
import ru.astrainteractive.klibs.rules.gradle.util.hasKotlinPlugin

internal fun Project.kotlinJvmTarget(): String? {
    if (!hasKotlinPlugin()) return null
    val extension = extensions.findByType(KotlinJvmProjectExtension::class.java) ?: return null
    return extension.compilerOptions.jvmTarget.orNull?.target
}
