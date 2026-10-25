package ru.astrainteractive.klibs.rules.structure.unit.model

import java.io.File

/**
 * A build unit as its own build sees it.
 *
 * @property path the Gradle project path, `:modules:jail:impl`
 * @property isEntryPoint whether the unit holds the platform main class and the composition root
 * @property isRoot whether the unit is the root project of the build
 * @property projectDependencies paths of the project units the production code depends on
 * @property isLibrary whether the unit is a published library, whose dependencies are not product-unit boundaries
 */
public class BuildUnit(
    public val path: String,
    public val directory: File,
    public val isEntryPoint: Boolean,
    public val isRoot: Boolean,
    public val projectDependencies: Set<String>,
    public val isLibrary: Boolean = false
) {
    /**
     * The last segment of [path]: `impl` for `:modules:jail:impl`.
     */
    public val name: String
        get() = path.substringAfterLast(':')
}
