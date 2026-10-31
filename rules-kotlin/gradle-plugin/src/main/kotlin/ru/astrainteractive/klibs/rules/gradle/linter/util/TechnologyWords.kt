package ru.astrainteractive.klibs.rules.gradle.linter.util

private val TECHNOLOGY_WORD = Regex("[a-z][a-z0-9]{2,}")

private val CAMEL_CASE_HEAD = Regex("^[A-Z]?[a-z0-9]+")

private val NON_TECHNOLOGY_WORDS = setOf(
    "android",
    "annotation",
    "api",
    "bom",
    "command",
    "common",
    "component",
    "composable",
    "compose",
    "context",
    "core",
    "database",
    "di",
    "entity",
    "error",
    "event",
    "expansion",
    "foundation",
    "gradle",
    "gui",
    "job",
    "jvm",
    "kotlin",
    "kotlinx",
    "lifecycle",
    "material",
    "material3",
    "menu",
    "migration",
    "model",
    "plugin",
    "route",
    "runtime",
    "screen",
    "service",
    "slot",
    "stub",
    "table",
    "test",
    "theme",
    "ui",
    "usecase",
    "util",
    "viewmodel",
    "warmup",
)

/**
 * The technology words a unit's libraries name, so that a package named after one (`mongodb`) is a technology
 * package: the last segment of each group and the first word of each artifact, whole and up to its first camel-case
 * boundary (`VaultAPI` gives `vaultapi` and `vault`), except the words of kinds and the generic ones (`core`,
 * `runtime`, `kotlinx`).
 *
 * @param coordinates the external dependencies of the unit as `group:artifact`
 */
internal fun findTechnologyWords(coordinates: List<String>): Set<String> {
    return coordinates
        .flatMap { coordinate ->
            val group = coordinate.substringBefore(':').substringAfterLast('.')
            val artifact = coordinate.substringAfter(':').substringBefore('-')
            listOfNotNull(
                group,
                artifact,
                CAMEL_CASE_HEAD.find(artifact)?.value
            )
        }
        .map(String::lowercase)
        .filter(TECHNOLOGY_WORD::matches)
        .filterNot(NON_TECHNOLOGY_WORDS::contains)
        .toSet()
}
