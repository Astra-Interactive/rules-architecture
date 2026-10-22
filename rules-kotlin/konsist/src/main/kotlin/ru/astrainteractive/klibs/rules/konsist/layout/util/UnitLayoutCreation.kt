package ru.astrainteractive.klibs.rules.konsist.layout.util

import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitLayout

private fun commonPrefix(packages: Collection<String>): List<String> {
    val split = packages.map { packageName -> packageName.split('.') }
    val shortest = split.minOf(List<String>::size)
    return (0 until shortest)
        .takeWhile { index -> split.all { segments -> segments[index] == split.first()[index] } }
        .map { index -> split.first()[index] }
}

/**
 * The segment sequences the unit path may leave in the root package, most specific first. The last path
 * segment is tried whole, joined and with trailing parts dropped (`integration-papi` gives
 * `[integration, papi]`, `[integrationpapi]`, `[integration]`), each first with its parent segments in
 * front (`:modules:data:dao` tries `[data, dao]` before `[dao]`).
 */
private fun unitNameCandidates(unitPath: String): List<List<String>> {
    val pathSegments = unitPath.split(':').filter(String::isNotEmpty)
    val parents = pathSegments.dropLast(1)
    val parts = pathSegments.lastOrNull().orEmpty().split('-').filter(String::isNotEmpty)
    val shortened = (parts.size - 1 downTo 1).map(parts::take)
    val tails = listOf(parts, listOf(parts.joinToString(""))) + shortened
    return tails.filter(List<String>::isNotEmpty).flatMap { tail ->
        (parents.size downTo 0).map { count -> parents.takeLast(count) + tail }
    }
}

private fun firstEndOf(prefix: List<String>, sequence: List<String>): Int? {
    return (0..prefix.size - sequence.size)
        .firstOrNull { start -> prefix.subList(start, start + sequence.size) == sequence }
        ?.let { start -> start + sequence.size }
}

private fun detectRootPackage(packages: Collection<String>, unitPath: String?): String {
    if (packages.isEmpty()) return ""
    val prefix = commonPrefix(packages)
    val unitEnd = unitPath?.let { path ->
        unitNameCandidates(path).firstNotNullOfOrNull { sequence -> firstEndOf(prefix, sequence) }
    }
    if (unitEnd != null) return prefix.take(unitEnd).joinToString(".")
    val lastKind = prefix.lastOrNull()?.let(Kind::fromSegment)
    val isSingleKindPackage = packages.toSet().size == 1 && lastKind != null
    val root = if (isSingleKindPackage) prefix.dropLast(1) else prefix
    return root.joinToString(".")
}

/**
 * @param rootPackage the unit's root package; when `null`, the common package of [packages] cut after
 * the segments that match the unit's name in [unitPath] (`:modules:event-bukkit` ends at `event`)
 */
internal fun createUnitLayout(
    packages: Collection<String>,
    rootPackage: String?,
    unitPath: String?,
    unitKind: UnitKind
): UnitLayout {
    val root = rootPackage ?: detectRootPackage(packages, unitPath)
    return UnitLayout(root, unitKind)
}
