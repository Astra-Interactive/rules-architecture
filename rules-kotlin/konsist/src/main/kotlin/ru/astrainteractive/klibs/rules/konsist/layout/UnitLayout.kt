package ru.astrainteractive.klibs.rules.konsist.layout

/**
 * The package layout of one build unit: its root package and the modules of its context.
 *
 * A sub-package of the root counts as a module only when it holds a role package of its own; an unknown
 * sub-package (an old `presentation`) has no placement, so no check judges its files.
 */
internal class UnitLayout(
    val rootPackage: String,
    private val modules: Set<String>
) {
    private fun segmentsBelowRoot(packageName: String): List<String>? {
        if (packageName == rootPackage) return emptyList()
        if (!packageName.startsWith("$rootPackage.")) return null
        return packageName.removePrefix("$rootPackage.").split('.')
    }

    fun placementOf(packageName: String): Placement? {
        val segments = segmentsBelowRoot(packageName) ?: return null
        val first = segments.firstOrNull() ?: return Placement(module = null, role = null, subpackages = emptyList())
        val firstRole = Role.fromSegment(first)
        if (firstRole != null) return Placement(module = null, role = firstRole, subpackages = segments.drop(1))
        if (first !in modules) return null
        val second = segments.getOrNull(1) ?: return Placement(module = first, role = null, subpackages = emptyList())
        val secondRole = Role.fromSegment(second) ?: return null
        return Placement(module = first, role = secondRole, subpackages = segments.drop(2))
    }

    companion object {
        private fun commonPrefix(packages: Collection<String>): List<String> {
            val split = packages.map { packageName -> packageName.split('.') }
            val shortest = split.minOf { segments -> segments.size }
            return (0 until shortest)
                .takeWhile { index -> split.all { segments -> segments[index] == split.first()[index] } }
                .map { index -> split.first()[index] }
        }

        private fun detectRootPackage(packages: Collection<String>): String {
            if (packages.isEmpty()) return ""
            val prefix = commonPrefix(packages).toMutableList()
            while (prefix.isNotEmpty() && Role.fromSegment(prefix.last()) != null) {
                prefix.removeAt(prefix.lastIndex)
            }
            return prefix.joinToString(".")
        }

        private fun detectModules(rootPackage: String, packages: Collection<String>): Set<String> = packages
            .filter { packageName -> packageName.startsWith("$rootPackage.") }
            .map { packageName -> packageName.removePrefix("$rootPackage.").split('.') }
            .filter { segments -> segments.size >= 2 && Role.fromSegment(segments[0]) == null }
            .filter { segments -> Role.fromSegment(segments[1]) != null }
            .map { segments -> segments[0] }
            .toSet()

        /**
         * @param rootPackage the unit's root package; detected from [packages] when `null`, as their common
         * prefix without trailing role segments
         */
        fun create(packages: Collection<String>, rootPackage: String?): UnitLayout {
            val root = rootPackage ?: detectRootPackage(packages)
            return UnitLayout(root, detectModules(root, packages))
        }
    }
}
