package ru.astrainteractive.klibs.rules.konsist.check.declaration

import com.lemonappdev.konsist.api.declaration.KoClassDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.isFacade
import ru.astrainteractive.klibs.rules.konsist.check.model.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.Layer
import ru.astrainteractive.klibs.rules.konsist.layout.Placement
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a `Default<Context>Api` that does not implement `<Context>Api` or that sits outside the context root,
 * where it delegates to the use cases next to it.
 */
internal class DefaultApiImplementsApi : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = setOf(UnitKind.CONTEXT)

    private fun findViolations(facade: KoClassDeclaration, placement: Placement?): List<String> = buildList {
        val apiName = facade.name.removePrefix(DEFAULT_PREFIX)
        if (facade.parents().none { parent -> parent.name == apiName }) {
            add("${facade.name} must implement $apiName, the interface it is the one implementation of.")
        }
        if (placement != null && placement.layer != Layer.USE_CASE) {
            add("Move ${facade.name} to the context root, next to the use cases it delegates to.")
        }
    }

    override fun check(sources: UnitSources): List<Violation> = sources.mainFiles.flatMap { file ->
        val placement = sources.placementOf(file)
        file.topLevelClasses()
            .filter { declaration -> declaration.isFacade() }
            .flatMap { facade ->
                findViolations(facade, placement).map { message ->
                    Violation(RULE, sources.relativePath(file), facade.name, message)
                }
            }
    }

    private companion object {
        const val RULE = "DefaultApiImplementsApi"
        const val DEFAULT_PREFIX = "Default"
    }
}
