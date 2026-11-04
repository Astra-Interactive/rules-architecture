package ru.astrainteractive.klibs.rules.konsist.ownership.konsist

import com.lemonappdev.konsist.api.Konsist
import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.provider.KoNameProvider
import com.lemonappdev.konsist.api.provider.KoTextProvider
import ru.astrainteractive.klibs.rules.core.unit.model.UnitContext
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.model.util.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelNames
import ru.astrainteractive.klibs.rules.konsist.ownership.model.OwnershipDeclaration
import ru.astrainteractive.klibs.rules.konsist.ownership.model.OwnershipUnit
import java.io.File

internal class KonsistOwnershipReport {
    private fun directories(unit: OwnershipUnit): List<File> {
        return unit.context.compilations.filterNot(UnitContext.Compilation::test)
            .flatMap { compilation -> compilation.sourceSets.values.flatten() }
            .distinct().sorted().map(::File)
    }

    private fun loadFiles(unit: OwnershipUnit): List<KoFileDeclaration> {
        val existing = directories(unit).filter(File::isDirectory)
        if (existing.isEmpty()) return emptyList()
        return Konsist.scopeFromExternalDirectories(existing.map(File::getAbsolutePath)).files
    }

    private fun qualify(file: KoFileDeclaration, name: String): String {
        return "${file.packageName}.$name".trimStart('.')
    }

    private fun declarations(
        unit: OwnershipUnit,
        file: KoFileDeclaration,
        references: KotlinOwnershipReferences
    ): List<OwnershipDeclaration> {
        val named = file.classes(includeNested = false, includeLocal = false) +
            file.interfaces(includeNested = false) + file.objects(includeNested = false) +
            file.functions(includeNested = false, includeLocal = false) +
            file.properties(includeNested = false) + file.typeAliases
        return named.map { declaration ->
            val name = qualify(file, (declaration as KoNameProvider).name)
            val text = (declaration as KoTextProvider).text
            val line = file.text.take(
                file.text.indexOf(text).coerceAtLeast(0)
            ).count { char -> char == '\n' } + 1
            val relative = File(file.path).relativeToOrSelf(unit.directory).invariantSeparatorsPath
            val wiring = "di" in file.packageName.split('.')
            val resolved = references.resolve(file, text)
            OwnershipDeclaration(
                name,
                unit,
                "$relative:$line",
                resolved.names - name,
                wiring,
                resolved.ambiguous
            )
        }
    }

    fun render(units: List<OwnershipUnit>): String {
        val files = units.associateWith(::loadFiles)
        val names = files.values.flatten()
            .flatMap { file -> file.topLevelNames().map { name -> qualify(file, name) } }.toSet()
        val references = KotlinOwnershipReferences(names)
        val declarations = files.flatMap { (unit, sources) ->
            sources.flatMap { file ->
                declarations(
                    unit,
                    file,
                    references
                )
            }
        }
        val limitations = buildList {
            add(
                "Kotlin lexical analysis: aliases, inheritance and signatures are included; overloads and shadowing " +
                    "are not compiler-resolved. All references of shared API declarations are retained conservatively."
            )
            add(
                "Generated sources, Java, included builds, reflection and consumers outside participating units " +
                    "are not resolved. Inspect them before accepting any candidate."
            )
            units.sortedBy(OwnershipUnit::path).forEach { unit ->
                add(
                    "Participating unit ${unit.path}: ${files.getValue(unit).size} Kotlin files; " +
                        "production roots: ${directories(unit).joinToString()}"
                )
                if (unit.context.compilations.isEmpty()) {
                    add("${unit.path}: no Kotlin compilation metadata; sources were not inferred from directory names.")
                }
                val missing = directories(unit).filterNot(File::isDirectory)
                missing.forEach { directory -> add("${unit.path}: absent source directory $directory") }
                val unknown = unit.context.compilations.filterNot(UnitContext.Compilation::test)
                    .flatMap { compilation -> compilation.projectDependencies }
                    .filter { dependency -> units.none { candidate -> candidate.path == dependency } }
                unknown.forEach { dependency -> add("${unit.path}: missing unit metadata for $dependency") }
                if (unit.context.kind == UnitKind.API && unit.context.publishedLibrary) {
                    add("${unit.path}: published API retained regardless of observed consumers.")
                }
            }
        }
        return ApiOwnershipReport().render(declarations, limitations)
    }
}
