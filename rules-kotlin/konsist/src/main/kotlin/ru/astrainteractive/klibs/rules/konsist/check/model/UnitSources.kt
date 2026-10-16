package ru.astrainteractive.klibs.rules.konsist.check.model

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.layout.Placement
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitLayout
import java.io.File

/**
 * The parsed sources of one build unit, with the layout its checks judge them by.
 */
internal class UnitSources(
    val kind: UnitKind,
    val layout: UnitLayout,
    val mainFiles: List<KoFileDeclaration>,
    val testFiles: List<KoFileDeclaration>,
    private val projectDirectory: File
) {
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

    private fun qualify(packageName: String, name: String): String =
        if (packageName.isEmpty()) name else "$packageName.$name"

    fun relativePath(file: KoFileDeclaration): String =
        File(file.path).relativeTo(projectDirectory).invariantSeparatorsPath

    fun placementOf(file: KoFileDeclaration): Placement? = layout.placementOf(file.packageName)

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
        private fun parse(directories: List<File>): List<KoFileDeclaration> {
            val existing = directories.filter { directory -> directory.isDirectory }
            if (existing.isEmpty()) return emptyList()
            return Konsist.scopeFromExternalDirectories(existing.map { directory -> directory.absolutePath }).files
        }

        fun load(
            kind: UnitKind,
            rootPackage: String?,
            mainDirectories: List<File>,
            testDirectories: List<File>,
            projectDirectory: File
        ): UnitSources {
            val mainFiles = parse(mainDirectories)
            val layout = UnitLayout.create(mainFiles.map { file -> file.packageName }.distinct(), rootPackage)
            return UnitSources(kind, layout, mainFiles, parse(testDirectories), projectDirectory)
        }
    }
}
