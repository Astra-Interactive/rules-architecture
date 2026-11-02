package ru.astrainteractive.klibs.rules.structure.graph.structure

import ru.astrainteractive.klibs.rules.core.unit.model.ImplementationAccess
import ru.astrainteractive.klibs.rules.core.unit.model.UiComposition
import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit
import ru.astrainteractive.klibs.rules.structure.unit.structure.isPlatformUnitName
import ru.astrainteractive.klibs.rules.structure.unit.structure.isTechnologyUnitName
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind as Kind

internal object UnitDependencyDirection {
    private const val CORE_NAME = "core"
    private const val DESIGN_SYSTEM_NAME = "ui"

    private fun coreKindOf(segments: List<String>): Kind {
        val name = segments.last()
        val isDesignSystem = DESIGN_SYSTEM_NAME in segments.dropWhile { segment -> segment != CORE_NAME }
        return when {
            isPlatformUnitName(name) -> Kind.CORE_PLATFORM
            isDesignSystem -> Kind.CORE_UI
            isTechnologyUnitName(name) -> Kind.TECHNOLOGY
            else -> Kind.CORE_API
        }
    }

    private fun kindOf(path: String, isEntryPoint: Boolean): Kind {
        val segments = path.split(':')
        val name = segments.last()
        val isCore = CORE_NAME in segments
        if (isEntryPoint || "instances" in segments) return Kind.ENTRY_POINT
        if (isCore) return coreKindOf(segments)
        return when {
            name == "api" -> Kind.API
            name == "impl" -> Kind.IMPL
            name == "contract" -> Kind.CONTRACT
            name == "dao" -> Kind.STORAGE_CONTRACT
            name == "fake" -> Kind.FAKE
            isPlatformUnitName(name) -> Kind.PLATFORM
            isTechnologyUnitName(name) -> Kind.TECHNOLOGY
            else -> Kind.CONTEXT
        }
    }

    fun kindOf(unit: BuildUnit): Kind = unit.context?.kind ?: kindOf(unit.path, unit.isEntryPoint)

    fun targetKind(unit: BuildUnit, dependency: String): Kind {
        return unit.dependencyContexts[dependency]?.kind ?: kindOf(dependency, isEntryPoint = false)
    }

    private fun allowedTargets(source: Kind): Set<Kind> {
        return when (source) {
            Kind.ENTRY_POINT -> Kind.entries.toSet() - Kind.FAKE - Kind.ENTRY_POINT
            Kind.PLATFORM, Kind.CONTEXT -> setOf(
                Kind.API,
                Kind.CORE,
                Kind.CORE_API,
                Kind.CORE_PLATFORM,
                Kind.CORE_UI,
                Kind.STORAGE_CONTRACT,
                Kind.CONTRACT
            )
            Kind.API -> setOf(
                Kind.API,
                Kind.CORE,
                Kind.CORE_API,
                Kind.STORAGE_CONTRACT,
                Kind.CONTRACT
            )
            Kind.IMPL -> setOf(
                Kind.API,
                Kind.CORE,
                Kind.CORE_API,
                Kind.CORE_UI,
                Kind.CONTRACT
            )
            Kind.TECHNOLOGY -> setOf(
                Kind.STORAGE_CONTRACT,
                Kind.CONTRACT,
                Kind.CORE,
                Kind.CORE_API
            )
            Kind.STORAGE_CONTRACT -> setOf(Kind.CORE_API)
            Kind.CORE_API, Kind.CONTRACT -> emptySet()
            Kind.CORE_PLATFORM -> setOf(Kind.CORE_API)
            Kind.CORE_UI -> setOf(
                Kind.CORE,
                Kind.CORE_API,
                Kind.CORE_UI
            )
            Kind.FAKE -> setOf(Kind.API)
            Kind.IMPL_UI -> setOf(
                Kind.API,
                Kind.CORE,
                Kind.CORE_API,
                Kind.CORE_PLATFORM,
                Kind.CORE_UI,
                Kind.STORAGE_CONTRACT,
                Kind.CONTRACT
            )
            Kind.CORE -> emptySet()
        }
    }

    private fun isSameImmediateParent(source: String, target: String): Boolean {
        return source.substringBeforeLast(':') == target.substringBeforeLast(':')
    }

    private fun isAllowedByUiPolicy(unit: BuildUnit, dependency: String, target: Kind): Boolean {
        val policy = unit.context?.policy ?: return false
        if (kindOf(unit) != Kind.IMPL_UI) return false
        if (target == Kind.IMPL && policy.implementationAccess == ImplementationAccess.SIBLINGS) {
            return isSameImmediateParent(unit.path, dependency)
        }
        return target == Kind.IMPL_UI && policy.composition == UiComposition.HOST
    }

    fun validateExceptions(unit: BuildUnit) {
        unit.context?.policy?.checks?.allowedDependencies.orEmpty().forEach { (dependency, reason) ->
            require(reason.isNotBlank()) { "Dependency exception for $dependency in ${unit.path} needs a reason" }
            require(kindOf(unit) != Kind.FAKE) {
                "Fake unit ${unit.path} cannot declare dependency exceptions: it depends only on its own api unit"
            }
            require(dependency in unit.projectDependencies) {
                "Dependency exception for $dependency in ${unit.path} has no declared production edge"
            }
            val target = targetKind(unit, dependency)
            require(target != Kind.FAKE && target != Kind.ENTRY_POINT) {
                "Dependency exception for $dependency in ${unit.path} cannot target role $target"
            }
        }
    }

    fun findForbidden(unit: BuildUnit): List<String> {
        validateExceptions(unit)
        val source = kindOf(unit)
        val published = unit.context?.publishedLibrary ?: unit.isLibrary
        val core = source in setOf(
            Kind.CORE,
            Kind.CORE_API,
            Kind.CORE_PLATFORM,
            Kind.CORE_UI
        )
        val allowed = if (published && !core) {
            Kind.entries.toSet() - Kind.FAKE - Kind.ENTRY_POINT
        } else {
            allowedTargets(source)
        }
        val ownContext = unit.path.substringBeforeLast(':')
        return unit.projectDependencies
            .filter { dependency ->
                val target = targetKind(unit, dependency)
                if (source == Kind.FAKE) {
                    target != Kind.API || dependency.substringBeforeLast(':') != ownContext
                } else {
                    val explicitlyAllowed = dependency in unit.context?.policy?.checks?.allowedDependencies.orEmpty()
                    val uiPolicyAllows = isAllowedByUiPolicy(
                        unit,
                        dependency,
                        target
                    )
                    target !in allowed && !uiPolicyAllows && !explicitlyAllowed
                }
            }
            .sorted()
    }

    fun message(unit: BuildUnit, dependency: String): String {
        val source = kindOf(unit)
        val target = targetKind(unit, dependency)
        val coreKinds = setOf(
            Kind.CORE,
            Kind.CORE_API,
            Kind.CORE_PLATFORM,
            Kind.CORE_UI
        )
        val isCoreDependency = target in coreKinds
        return when {
            target == Kind.FAKE ->
                "Drop the production dependency on $dependency: shared fakes are consumed only by tests."
            target == Kind.ENTRY_POINT -> "Drop $dependency: nothing depends on an entry-point unit."
            source == Kind.CONTRACT -> "A contract unit depends on nothing of the project: drop $dependency."
            source in coreKinds && !isCoreDependency -> "core depends on no feature unit: drop $dependency."
            target == Kind.IMPL && source == Kind.IMPL_UI ->
                "${unit.path} may depend on an impl unit only when sibling implementation access is enabled " +
                    "and both Gradle paths have the same immediate parent: drop $dependency or declare an exact exception."
            target == Kind.IMPL_UI && source == Kind.IMPL_UI ->
                "${unit.path} may depend on another UI implementation only in HOST composition mode: drop $dependency."
            target == Kind.IMPL && source == Kind.API ->
                "An api unit never depends on an impl unit: depend on the api unit next to $dependency."
            target == Kind.IMPL ->
                "Only the entry point depends on $dependency: depend on the api unit next to it."
            target == Kind.PLATFORM ->
                "Only the entry point depends on the platform unit $dependency: depend on the api unit next to it."
            else ->
                "The dependency $dependency is not allowed for ${unit.path} by the unit direction table: depend " +
                    "on an api unit, platform-free core or a storage contracts unit instead."
        }
    }
}
