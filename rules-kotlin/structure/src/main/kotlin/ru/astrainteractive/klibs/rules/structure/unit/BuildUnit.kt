package ru.astrainteractive.klibs.rules.structure.unit

import java.io.File

/**
 * A build unit as its own build sees it.
 *
 * @property path the Gradle project path, `:modules:jail:impl`
 * @property isEntryPoint whether the unit holds the platform main class and the composition root
 * @property isRoot whether the unit is the root project of the build
 * @property projectDependencies paths of the project units the production code depends on
 */
public data class BuildUnit(
    val path: String,
    val directory: File,
    val isEntryPoint: Boolean,
    val isRoot: Boolean,
    val projectDependencies: Set<String>
) {
    /**
     * The last segment of [path]: `impl` for `:modules:jail:impl`.
     */
    val name: String
        get() = path.substringAfterLast(':')
}
