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
     * The kind is the last kind word of the package, except Compose's `composable`, which is the kind whenever it
     * appears: the segment below it is its sub-kind, even when that segment is a kind word too (`screen`).
     *
     * @return `null` for a package outside the unit's root package
     */
    fun placementOf(packageName: String): Placement? {
        val segments = segmentsBelowRoot(packageName) ?: return null
        val composableIndex = segments.indexOf(Kind.COMPOSABLE_WORD)
        val kindIndex = if (composableIndex >= 0) {
            composableIndex
        } else {
            segments.indexOfLast { segment -> Kind.fromSegment(segment) != null }
        }
        if (kindIndex < 0) {
            val kind = Kind.IMPLEMENTATION.takeIf { unitKind == UnitKind.TECHNOLOGY && segments.isNotEmpty() }
            return Placement(
                concepts = segments,
                kind = kind,
                kindWord = null,
                subKind = null,
                below = emptyList()
            )
        }
        val subKind = segments.getOrNull(kindIndex + 1)?.takeIf { kindIndex == composableIndex }
        val belowIndex = if (subKind == null) kindIndex + 1 else kindIndex + 2
        return Placement(
            concepts = segments.take(kindIndex),
            kind = Kind.fromSegment(segments[kindIndex]),
            kindWord = segments[kindIndex],
            subKind = subKind,
            below = segments.drop(belowIndex)
        )
    }
}
