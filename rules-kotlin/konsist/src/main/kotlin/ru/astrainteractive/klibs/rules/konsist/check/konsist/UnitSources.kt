package ru.astrainteractive.klibs.rules.konsist.check.konsist

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.model.util.importedName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelNames
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.model.Placement
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitLayout
import java.io.File

/**
 * The parsed sources of one build unit, with the layout its checks judge them by.
 *
 * @property unitPath the Gradle path of the unit (`:modules:core:api`), `null` when it was not given
 * @property mainDirectories the main source directories as given, one per source set
 */
internal class UnitSources(
    val kind: UnitKind,
    val layout: UnitLayout,
    val unitPath: String?,
    val mainDirectories: List<File>,
    val mainFiles: List<KoFileDeclaration>,
    val testFiles: List<KoFileDeclaration>,
    private val mainSourceSets: Map<String, File>,
    private val testSourceSets: Map<String, File>,
    private val projectDirectory: File
) {
    /**
     * Whether the unit is a Compose UI unit: a main file outside `model` imports Compose, so its build depends
     * on it. A `model` import alone does not count, as it is the import this fact judges.
     */
    val isComposeUnit: Boolean by lazy {
        mainFiles
            .filterNot { file -> layout.placementOf(file.packageName)?.kind == Kind.MODEL }
            .any { file -> file.imports.any { import -> isComposeImport(import.importedName) } }
    }

    /**
     * The package of every top-level declaration of the main sources, by its fully qualified name.
     */
    private val packageByQualifiedName: Map<String, String> by lazy {
        buildMap {
            mainFiles.forEach { file ->
                file.topLevelNames().forEach { name -> put(qualify(file.packageName, name), file.packageName) }
            }
        }
    }

    private fun qualify(packageName: String, name: String): String {
        return if (packageName.isEmpty()) name else "$packageName.$name"
    }

    fun relativePath(file: KoFileDeclaration): String {
        return File(file.path).relativeTo(projectDirectory).invariantSeparatorsPath
    }

    fun placementOf(file: KoFileDeclaration): Placement? = layout.placementOf(file.packageName)

    private fun sourceSetOf(file: KoFileDeclaration, sourceSets: Map<String, File>): String? {
        return sourceSets.entries.singleOrNull { entry -> File(file.path).startsWith(entry.value) }?.key
    }

    fun mainSourceSetOf(file: KoFileDeclaration): String? = sourceSetOf(file, mainSourceSets)

    fun testSourceSetOf(file: KoFileDeclaration): String? = sourceSetOf(file, testSourceSets)

    /**
     * Resolves an import of the main sources to the package that declares it, trying shorter prefixes for an
     * import of a nested declaration; `null` for anything declared outside the unit.
     */
    fun packageOfImport(importName: String): String? {
        if (importName.endsWith(".*")) {
            return importName.removeSuffix(".*").takeIf { packageName -> layout.placementOf(packageName) != null }
        }
        return generateSequence(importName) { name -> name.substringBeforeLast('.', "").ifEmpty { null } }
            .firstNotNullOfOrNull { name -> packageByQualifiedName[name] }
    }

    companion object {
        private val COMPOSE_PACKAGE_PREFIXES = listOf("androidx.compose.", "org.jetbrains.compose.")

        fun isComposeImport(importedName: String): Boolean {
            return COMPOSE_PACKAGE_PREFIXES.any(importedName::startsWith)
        }

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
        @Suppress("CompanionFactoryFunction")
        internal fun createUnitLayout(
            packages: Collection<String>,
            rootPackage: String?,
            unitPath: String?,
            unitKind: UnitKind
        ): UnitLayout {
            val root = rootPackage ?: detectRootPackage(packages, unitPath)
            return UnitLayout(root, unitKind)
        }

        private fun parse(directories: List<File>): List<KoFileDeclaration> {
            val existing = directories.filter(File::isDirectory)
            if (existing.isEmpty()) return emptyList()
            return Konsist.scopeFromExternalDirectories(existing.map(File::getAbsolutePath)).files
        }

        /**
         * @param unitPath the Gradle path of the unit, used to find its root package when [rootPackage] is `null`
         */
        fun load(
            kind: UnitKind,
            rootPackage: String?,
            unitPath: String?,
            mainDirectories: List<File>,
            testDirectories: List<File>,
            mainSourceSets: Map<String, File>,
            testSourceSets: Map<String, File>,
            projectDirectory: File
        ): UnitSources {
            val mainFiles = parse(mainDirectories)
            val packages = mainFiles.map(KoFileDeclaration::packageName).distinct()
            val layout = createUnitLayout(
                packages,
                rootPackage,
                unitPath,
                kind
            )
            return UnitSources(
                kind = kind,
                layout = layout,
                unitPath = unitPath,
                mainDirectories = mainDirectories,
                mainFiles = mainFiles,
                testFiles = parse(testDirectories),
                mainSourceSets = mainSourceSets,
                testSourceSets = testSourceSets,
                projectDirectory = projectDirectory
            )
        }
    }
}
