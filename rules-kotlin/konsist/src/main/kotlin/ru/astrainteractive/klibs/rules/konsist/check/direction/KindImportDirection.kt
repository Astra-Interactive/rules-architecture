package ru.astrainteractive.klibs.rules.konsist.check.direction

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.importedName
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Checks the direction table of `arch-context.md` through the imports between the kind packages of a unit:
 * `model` depends on nothing of its level, contracts on `model`, use cases on contracts and `model`,
 * implementations on the contracts they implement, entry points on use cases and contracts, `theme` on `model`,
 * and nothing depends on `di`. A use case also reaches `util`, where the rules put the extensions it calls;
 * `util` and the entry points also reach `theme`, which only a Compose unit has.
 *
 * Imports that leave the unit, and packages of no known kind, are not judged.
 */
internal class KindImportDirection : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun allowedTargets(from: Kind): Set<Kind> = when (from) {
        Kind.MODEL -> setOf(Kind.MODEL)
        Kind.CONTRACT -> setOf(Kind.MODEL, Kind.CONTRACT)
        Kind.USECASE -> setOf(Kind.MODEL, Kind.CONTRACT, Kind.UTIL)
        Kind.UTIL -> setOf(Kind.MODEL, Kind.CONTRACT, Kind.UTIL, Kind.THEME)
        Kind.IMPLEMENTATION -> setOf(Kind.MODEL, Kind.CONTRACT, Kind.UTIL, Kind.IMPLEMENTATION)
        Kind.ENTRY_POINT -> setOf(Kind.MODEL, Kind.CONTRACT, Kind.UTIL, Kind.USECASE, Kind.ENTRY_POINT, Kind.THEME)
        Kind.THEME -> setOf(Kind.MODEL, Kind.THEME)
        Kind.DI -> Kind.entries.toSet()
    }

    private fun checkFile(sources: UnitSources, file: KoFileDeclaration): List<Violation> {
        val from = sources.placementOf(file) ?: return emptyList()
        val fromKind = from.kind ?: return emptyList()
        return file.imports.mapNotNull { import ->
            val importedName = import.importedName
            val targetPackage = sources.packageOfImport(importedName) ?: return@mapNotNull null
            val to = sources.layout.placementOf(targetPackage) ?: return@mapNotNull null
            val toKind = to.kind ?: return@mapNotNull null
            if (toKind in allowedTargets(fromKind)) return@mapNotNull null
            val message = "${from.describe()} must not import $importedName from ${to.describe()}: " +
                "a ${fromKind.name.lowercase()} kind depends on " +
                allowedTargets(fromKind).joinToString { kind -> kind.name.lowercase() } + " only."
            Violation(RULE, sources.relativePath(file), importedName, message)
        }
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles.flatMap { file -> checkFile(sources, file) }
    }

    private companion object {
        const val RULE = "KindImportDirection"
    }
}
