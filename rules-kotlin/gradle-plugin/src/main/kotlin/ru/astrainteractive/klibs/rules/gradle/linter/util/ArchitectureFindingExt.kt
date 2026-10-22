package ru.astrainteractive.klibs.rules.gradle.linter.util

import ru.astrainteractive.klibs.rules.gradle.linter.model.ArchitectureFinding

internal fun ArchitectureFinding.describe(): String = "$path: $message [$rule]"
