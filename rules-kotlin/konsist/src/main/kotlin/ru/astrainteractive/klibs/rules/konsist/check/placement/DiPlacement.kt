package ru.astrainteractive.klibs.rules.konsist.check.placement

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.packageName
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.Role
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a DI module (a class or object named `*Module`) outside a `di` package, a `di` package below a role
 * instead of at the level of a context or a module, and a second DI module in one `di` package.
 */
internal class DiPlacement : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private class DiModule(val file: KoFileDeclaration, val name: String)

    private fun diModules(file: KoFileDeclaration): List<DiModule> {
        val names = file.topLevelClasses().map { declaration -> declaration.name } +
            file.objects(includeNested = false).map { declaration -> declaration.name }
        return names
            .filter { name -> name.endsWith(MODULE_SUFFIX) }
            .map { name -> DiModule(file, name) }
    }

    private fun findPlacementMessage(sources: UnitSources, module: DiModule): String? {
        if (module.file.packageName.substringAfterLast('.') != DI_SEGMENT) {
            return "Move ${module.name} to the di package of the context or module it wires."
        }
        if (sources.kind != UnitKind.CONTEXT) return null
        val placement = sources.placementOf(module.file) ?: return null
        if (placement.role == Role.DI && placement.subpackages.isEmpty()) return null
        return "Move ${module.name} to the di package at the level of its context or module, not below a role."
    }

    private fun extraModules(modules: List<DiModule>): List<DiModule> = modules
        .filter { module -> module.file.packageName.substringAfterLast('.') == DI_SEGMENT }
        .groupBy { module -> module.file.packageName }
        .values
        .flatMap { modulesOfPackage -> modulesOfPackage.sortedBy { module -> module.name }.drop(1) }

    override fun check(sources: UnitSources): List<Violation> {
        val modules = sources.mainFiles.flatMap { file -> diModules(file) }
        val misplaced = modules.mapNotNull { module ->
            val message = findPlacementMessage(sources, module) ?: return@mapNotNull null
            Violation(RULE, sources.relativePath(module.file), module.name, message)
        }
        val extra = extraModules(modules).map { module ->
            val message = "Merge ${module.name} into the one DI module of ${module.file.packageName}."
            Violation(RULE, sources.relativePath(module.file), module.name, message)
        }
        return misplaced + extra
    }

    private companion object {
        const val RULE = "DiPlacement"
        const val MODULE_SUFFIX = "Module"
        const val DI_SEGMENT = "di"
    }
}
