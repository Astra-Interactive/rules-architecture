package ru.astrainteractive.klibs.rules.konsist.check.direction.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.sourceReferences
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelNames
import java.io.File

/**
 * Reports a package cycle only when every edge exists in the same compilation. Test compilations include their
 * associated production sources; incompatible platform source sets never contribute to one graph.
 */
internal class PackageDependencyCycle : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun compilationFiles(sources: UnitSources): List<List<KoFileDeclaration>> {
        val compilations = sources.context.compilations
        if (compilations.isEmpty()) return listOf(sources.mainFiles, sources.mainFiles + sources.testFiles)
        return compilations.map { compilation ->
            val directories = compilation.sourceSets.values.flatten().map { directory -> File(directory).toPath() }
            val files = if (compilation.test) sources.mainFiles + sources.testFiles else sources.mainFiles
            files.filter { file -> directories.any { directory -> File(file.path).toPath().startsWith(directory) } }
        }
    }

    private fun packageOfReference(reference: String, packages: Map<String, Set<String>>): String? {
        if (reference.endsWith(".*")) return reference.removeSuffix(".*").takeIf(packages::containsKey)
        return generateSequence(reference) { name -> name.substringBeforeLast('.', "").ifEmpty { null } }
            .firstNotNullOfOrNull { name ->
                val parent = name.substringBeforeLast('.', "")
                parent.takeIf { name.substringAfterLast('.') in packages[parent].orEmpty() }
            }
    }

    private fun dependencyGraph(files: List<KoFileDeclaration>): Map<String, Set<String>> {
        val packages = files.groupBy(KoFileDeclaration::packageName)
        val names = packages.mapValues { entry -> entry.value.flatMap(KoFileDeclaration::topLevelNames).toSet() }
        return packages.mapValues { entry ->
            entry.value.flatMap { file -> file.sourceReferences() }
                .mapNotNull { reference -> packageOfReference(reference, names) }
                .filterNot { target -> target == entry.key }
                .toSortedSet()
        }
    }

    private fun canonicalCycle(path: List<String>): String {
        val first = path.indices.minBy(path::get)
        val rotated = path.drop(first).plus(path.take(first))
        return rotated.plus(rotated.first()).joinToString(" -> ")
    }

    private fun visit(
        current: String,
        graph: Map<String, Set<String>>,
        path: List<String>,
        visited: MutableSet<String>
    ): List<String> {
        if (current in path) return listOf(canonicalCycle(path.drop(path.indexOf(current))))
        if (!visited.add(current)) return emptyList()
        return graph.getValue(current).flatMap { target ->
            visit(
                target,
                graph,
                path.plus(current),
                visited
            )
        }
    }

    private fun cycles(files: List<KoFileDeclaration>): List<String> {
        val graph = dependencyGraph(files)
        val visited = mutableSetOf<String>()
        return graph.keys.sorted().flatMap { source ->
            visit(
                source,
                graph,
                emptyList(),
                visited
            )
        }
    }

    override fun check(sources: UnitSources): List<Violation> {
        return compilationFiles(sources).flatMap(::cycles).distinct().sorted().map { cycle ->
            val first = cycle.substringBefore(" -> ")
            val file = sources.mainFiles.plus(sources.testFiles).first { candidate -> candidate.packageName == first }
            sources.createViolation(
                RULE,
                file,
                cycle,
                "Break the package dependency cycle $cycle: packages visible in one compilation form an acyclic graph."
            )
        }
    }

    private companion object {
        const val RULE = "PackageDependencyCycle"
    }
}
