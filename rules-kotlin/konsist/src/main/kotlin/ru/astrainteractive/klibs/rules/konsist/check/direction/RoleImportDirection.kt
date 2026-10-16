package ru.astrainteractive.klibs.rules.konsist.check.direction

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.importedName
import ru.astrainteractive.klibs.rules.konsist.check.model.isFacade
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.Layer
import ru.astrainteractive.klibs.rules.konsist.layout.Placement
import ru.astrainteractive.klibs.rules.konsist.layout.Role
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Checks the direction table of the cross-context rule through the imports between the roles of a context:
 * `model` depends on nothing, contracts on `model`, use cases on contracts and `model`, technology packages on
 * the contracts above them, entry points on use cases and contracts, and nothing depends on `di`.
 *
 * Imports that leave the unit, and packages with no placement, are not judged. `util` is not restricted.
 */
internal class RoleImportDirection : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = setOf(UnitKind.CONTEXT)

    private fun declaresFacade(file: KoFileDeclaration): Boolean =
        file.topLevelClasses().any { declaration -> declaration.isFacade() }

    private fun isAllowedFromTechnology(from: Placement, to: Placement): Boolean = when (to.layer) {
        Layer.MODEL, Layer.CONTRACT, Layer.MAPPING -> true
        Layer.USE_CASE -> from.role == Role.INTEGRATION
        Layer.TECHNOLOGY -> to.role == from.role && (to.technology == from.technology || to.technology == SHARED_MODEL)
        else -> false
    }

    private fun isAllowedFromEntryPoint(from: Placement, to: Placement): Boolean = when (to.layer) {
        Layer.MODEL, Layer.CONTRACT, Layer.USE_CASE -> true
        Layer.ENTRY_POINT -> to.role == from.role || (from.role == Role.COMPOSABLE && to.role == Role.VIEWMODEL)
        else -> false
    }

    private fun isAllowedFromUseCase(to: Placement, isFacade: Boolean): Boolean =
        to.layer == Layer.MODEL || to.layer == Layer.CONTRACT || (isFacade && to.layer == Layer.USE_CASE)

    private fun isAllowed(from: Placement, to: Placement, isFacade: Boolean): Boolean {
        if (to.layer == Layer.UTIL) return true
        return when (from.layer) {
            Layer.MODEL -> to.layer == Layer.MODEL
            Layer.CONTRACT -> to.layer == Layer.MODEL || to.layer == Layer.CONTRACT
            Layer.USE_CASE -> isAllowedFromUseCase(to, isFacade)
            Layer.MAPPING -> to.layer == Layer.MODEL || to.layer == Layer.CONTRACT || to.layer == Layer.MAPPING
            Layer.TECHNOLOGY -> isAllowedFromTechnology(from, to)
            Layer.ENTRY_POINT -> isAllowedFromEntryPoint(from, to)
            Layer.UTIL, Layer.DI -> true
        }
    }

    private fun checkFile(sources: UnitSources, file: KoFileDeclaration): List<Violation> {
        val from = sources.placementOf(file) ?: return emptyList()
        val isFacade = declaresFacade(file)
        return file.imports.mapNotNull { import ->
            val importedName = import.importedName
            val to = sources.packageOfImport(importedName)?.let(sources.layout::placementOf) ?: return@mapNotNull null
            if (isAllowed(from, to, isFacade)) return@mapNotNull null
            val message = "${from.describe().replaceFirstChar(Char::titlecase)} must not import $importedName " +
                "from ${to.describe()}."
            Violation(RULE, sources.relativePath(file), importedName, message)
        }
    }

    override fun check(sources: UnitSources): List<Violation> =
        sources.mainFiles.flatMap { file -> checkFile(sources, file) }

    private companion object {
        const val RULE = "RoleImportDirection"
        const val SHARED_MODEL = "model"
    }
}
