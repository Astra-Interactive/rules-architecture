package ru.astrainteractive.klibs.rules.structure.graph.structure

import ru.astrainteractive.klibs.rules.structure.unit.model.BuildUnit

internal object UnitDependencyDirection {
    private val platforms = setOf(
        "bukkit",
        "bungee",
        "fabric",
        "forge",
        "neoforge",
        "paper",
        "spigot",
        "velocity"
    )
    private val technologies = setOf(
        "cache4k",
        "datastore",
        "exposed",
        "file",
        "krate",
        "ktor",
        "memory",
        "okhttp",
        "retrofit",
        "room",
        "sqldelight"
    )

    private enum class Kind {
        ENTRY_POINT,
        LIBRARY,
        PLATFORM,
        CONTEXT,
        API,
        IMPL,
        TECHNOLOGY,
        STORAGE_CONTRACT,
        CONTRACT,
        CORE_API,
        CORE_PLATFORM,
        FAKE
    }

    private fun coreKindOf(name: String): Kind {
        return when (name) {
            in platforms -> Kind.CORE_PLATFORM
            in technologies -> Kind.TECHNOLOGY
            else -> Kind.CORE_API
        }
    }

    private fun kindOf(path: String, isEntryPoint: Boolean, isLibrary: Boolean = false): Kind {
        val segments = path.split(':')
        val name = segments.last()
        val isCore = "core" in segments
        if (isLibrary && !isCore) return Kind.LIBRARY
        if (isEntryPoint) return Kind.ENTRY_POINT
        if (isCore) return coreKindOf(name)
        return when (name) {
            "api" -> Kind.API
            "impl" -> Kind.IMPL
            "contract" -> Kind.CONTRACT
            "dao" -> Kind.STORAGE_CONTRACT
            "fake" -> Kind.FAKE
            in technologies -> Kind.TECHNOLOGY
            in platforms -> Kind.PLATFORM
            else -> Kind.CONTEXT
        }
    }

    private fun allowedTargets(source: Kind): Set<Kind> {
        return when (source) {
            Kind.ENTRY_POINT, Kind.LIBRARY -> Kind.entries.toSet()
            Kind.PLATFORM, Kind.CONTEXT -> setOf(
                Kind.API,
                Kind.CORE_API,
                Kind.CORE_PLATFORM,
                Kind.STORAGE_CONTRACT,
                Kind.CONTRACT
            )
            Kind.API -> setOf(
                Kind.API,
                Kind.CORE_API,
                Kind.STORAGE_CONTRACT,
                Kind.CONTRACT
            )
            Kind.IMPL -> setOf(Kind.API, Kind.CORE_API)
            Kind.TECHNOLOGY -> setOf(
                Kind.STORAGE_CONTRACT,
                Kind.CONTRACT,
                Kind.CORE_API
            )
            Kind.STORAGE_CONTRACT -> setOf(Kind.CORE_API)
            Kind.CORE_API, Kind.CONTRACT -> emptySet()
            Kind.CORE_PLATFORM -> setOf(Kind.CORE_API)
            Kind.FAKE -> setOf(Kind.API)
        }
    }

    fun findForbidden(unit: BuildUnit): List<String> {
        val source = kindOf(
            unit.path,
            unit.isEntryPoint,
            unit.isLibrary
        )
        val allowed = allowedTargets(source)
        val fakeApi = unit.path.removeSuffix(":fake").plus(":api")
        return unit.projectDependencies
            .filter { dependency -> source != Kind.FAKE || dependency != fakeApi }
            .filter { dependency -> source == Kind.FAKE || kindOf(dependency, isEntryPoint = false) !in allowed }
            .sorted()
    }

    fun message(unit: BuildUnit, dependency: String): String {
        val dependencyName = dependency.substringAfterLast(':')
        val isCoreDependency = "core" in dependency.split(':')
        return when {
            unit.name == "contract" -> "A contract unit depends on nothing of the project: drop $dependency."
            "core" in unit.path.split(':') && !isCoreDependency -> "core depends on no feature unit: drop $dependency."
            dependencyName == "impl" && unit.name == "api" ->
                "An api unit never depends on an impl unit: depend on the api unit next to $dependency."
            dependencyName == "impl" ->
                "Only the entry point depends on $dependency: depend on the api unit next to it."
            dependencyName in platforms && !isCoreDependency ->
                "Only the entry point depends on the platform unit $dependency: depend on the api unit next to it."
            else ->
                "The dependency $dependency is not allowed for ${unit.path} by the unit direction table: depend " +
                    "on an api unit, platform-free core or a storage contracts unit instead."
        }
    }
}
