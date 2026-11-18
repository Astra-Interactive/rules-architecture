package ru.astrainteractive.klibs.rules.gradle.linter.model

/**
 * @property catalogName the name of the catalog's type-safe accessor, `libs` for `gradle/libs.versions.toml`
 */
internal data class VersionCatalogSection(
    val catalogName: String,
    val table: Table,
    val aliases: List<String>
) {
    /**
     * @property accessorPrefix what follows the catalog name in the type-safe accessor of an alias of this table
     */
    internal enum class Table(val tomlName: String, val accessorPrefix: String) {
        LIBRARIES("libraries", ""),
        PLUGINS("plugins", "plugins."),
        BUNDLES("bundles", "bundles.")
    }
}
