package ru.astrainteractive.klibs.rules.gradle.linter.model

import org.gradle.api.file.Directory
import org.gradle.api.provider.Provider

internal class AndroidVariantSources(
    val name: String,
    val kotlinDirectories: Provider<out Collection<Directory>>
)
