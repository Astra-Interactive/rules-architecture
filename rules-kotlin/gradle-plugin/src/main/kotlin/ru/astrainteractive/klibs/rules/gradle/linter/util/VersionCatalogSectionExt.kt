package ru.astrainteractive.klibs.rules.gradle.linter.util

import ru.astrainteractive.klibs.rules.gradle.linter.model.ArchitectureFinding
import ru.astrainteractive.klibs.rules.gradle.linter.model.VersionCatalogSection

private const val ALIAS_PREFIX_RULE = "CatalogAliasPrefix"

private const val TOML_SEPARATOR = "-"

private const val ACCESSOR_SEPARATOR = "."

// Gradle builds the same accessor for each of these separators
private val ALIAS_SEPARATORS = Regex("[-_.]")

private fun splitAlias(alias: String): List<String> = alias.split(ALIAS_SEPARATORS)

private fun isProperPrefix(prefix: List<String>, segments: List<String>): Boolean {
    return segments.size > prefix.size && segments.subList(0, prefix.size) == prefix
}

/**
 * Moves the segments that tell [longer] apart in front of the last segment of [shorter]:
 * `decompose-compose` and `decompose-compose-experimental` give `decompose-experimental-compose`.
 */
private fun buildRename(shorter: List<String>, longer: List<String>): List<String> {
    return shorter.dropLast(1) + longer.drop(shorter.size) + shorter.last()
}

private fun VersionCatalogSection.buildAliasPrefixFinding(
    shorter: List<String>,
    longer: List<String>
): ArchitectureFinding {
    val shorterAlias = shorter.joinToString(TOML_SEPARATOR)
    val longerAlias = longer.joinToString(TOML_SEPARATOR)
    val accessor = catalogName + ACCESSOR_SEPARATOR + table.accessorPrefix + shorter.joinToString(ACCESSOR_SEPARATOR)
    val rename = buildRename(shorter, longer).joinToString(TOML_SEPARATOR)
    return ArchitectureFinding(
        rule = ALIAS_PREFIX_RULE,
        path = catalogName,
        subject = "${table.tomlName}: $shorterAlias, $longerAlias",
        message = "Rename the [${table.tomlName}] alias $longerAlias: $shorterAlias is a prefix of it, so Gradle " +
            "turns $accessor into a group, and every build script that uses $accessor as a dependency stops " +
            "compiling. Put the segment that tells it apart before the shared prefix ends, e.g. $rename."
    )
}

internal fun VersionCatalogSection.findAliasPrefixFindings(): List<ArchitectureFinding> {
    val segments = aliases.map(::splitAlias)
        .distinct()
        .sortedBy { alias -> alias.joinToString(TOML_SEPARATOR) }
    return segments.flatMap { shorter ->
        segments
            .filter { longer -> isProperPrefix(shorter, longer) }
            .map { longer -> buildAliasPrefixFinding(shorter, longer) }
    }
}
