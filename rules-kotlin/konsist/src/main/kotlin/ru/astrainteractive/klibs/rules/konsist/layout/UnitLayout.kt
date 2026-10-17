package ru.astrainteractive.klibs.rules.konsist.layout

/**
 * The package layout of one build unit: its root package and its kind.
 */
internal class UnitLayout(
    val rootPackage: String,
    val unitKind: UnitKind
) {
    fun segmentsBelowRoot(packageName: String): List<String>? {
        if (packageName == rootPackage) return emptyList()
        if (rootPackage.isEmpty()) return packageName.split('.')
        if (!packageName.startsWith("$rootPackage.")) return null
        return packageName.removePrefix("$rootPackage.").split('.')
    }

    /**
     * @return `null` for a package outside the unit's root package
     */
    fun placementOf(packageName: String): Placement? {
        val segments = segmentsBelowRoot(packageName) ?: return null
        val kindIndex = segments.indexOfLast { segment -> Kind.fromSegment(segment) != null }
        if (kindIndex < 0) {
            val kind = Kind.IMPLEMENTATION.takeIf { unitKind == UnitKind.TECHNOLOGY && segments.isNotEmpty() }
            return Placement(concepts = segments, kind = kind, kindWord = null, below = emptyList())
        }
        return Placement(
            concepts = segments.take(kindIndex),
            kind = Kind.fromSegment(segments[kindIndex]),
            kindWord = segments[kindIndex],
            below = segments.drop(kindIndex + 1)
        )
    }

    companion object {
        private fun commonPrefix(packages: Collection<String>): List<String> {
            val split = packages.map { packageName -> packageName.split('.') }
            val shortest = split.minOf { segments -> segments.size }
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
            val pathSegments = unitPath.split(':').filter { segment -> segment.isNotEmpty() }
            val parents = pathSegments.dropLast(1)
            val parts = pathSegments.lastOrNull().orEmpty().split('-').filter { part -> part.isNotEmpty() }
            val shortened = (parts.size - 1 downTo 1).map { size -> parts.take(size) }
            val tails = listOf(parts, listOf(parts.joinToString(""))) + shortened
            return tails.filter { tail -> tail.isNotEmpty() }.flatMap { tail ->
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
            val lastKind = prefix.lastOrNull()?.let { segment -> Kind.fromSegment(segment) }
            val isSingleKindPackage = packages.toSet().size == 1 && lastKind != null
            val root = if (isSingleKindPackage) prefix.dropLast(1) else prefix
            return root.joinToString(".")
        }

        /**
         * @param rootPackage the unit's root package; when `null`, the common package of [packages] cut after
         * the segments that match the unit's name in [unitPath] (`:modules:event-bukkit` ends at `event`)
         */
        fun create(
            packages: Collection<String>,
            rootPackage: String?,
            unitPath: String?,
            unitKind: UnitKind
        ): UnitLayout {
            val root = rootPackage ?: detectRootPackage(packages, unitPath)
            return UnitLayout(root, unitKind)
        }
    }
}
