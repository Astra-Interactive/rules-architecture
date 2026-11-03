package ru.astrainteractive.klibs.rules.konsist.check.direction.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.sourceReferences
import ru.astrainteractive.klibs.rules.konsist.check.model.util.topLevelClasses
import ru.astrainteractive.klibs.rules.konsist.layout.model.Kind

/**
 * Reports a platform, framework or ORM import in `model`, `api` and `usecase`, and anywhere in an `api` unit,
 * which holds platform-free code only. Mixed app core admits common Ktor client wiring in `di` and Android types in
 * Android-only `util` source sets. Platform and technology units keep their explicit model, contract and use-case
 * boundaries; their implementation packages may import their platform. In a Compose UI unit, `model` and the UI kinds
 * may import Compose: an `api` unit whose state holder declares `Render()` is such a unit.
 */
internal class PlatformImport : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    private fun isChecked(sources: UnitSources, file: KoFileDeclaration): Boolean {
        if (sources.kind == UnitKind.API || sources.kind == UnitKind.CORE_API || sources.kind == UnitKind.CORE) {
            return true
        }
        return sources.placementOf(file)?.kind in CHECKED_KINDS
    }

    private fun isAllowedCoreIntegration(sources: UnitSources, file: KoFileDeclaration, reference: String): Boolean {
        if (sources.kind != UnitKind.CORE) return false
        val sourceSet = sources.mainSourceSetOf(file) ?: return false
        val kind = sources.placementOf(file)?.kind
        val clientImport = reference.startsWith("io.ktor.client.")
        val engineImport = reference.startsWith("io.ktor.client.engine.")
        val client = clientImport && !engineImport
        val engineApi = reference.removePrefix("io.ktor.client.engine.")
        val commonEngineApi = engineImport && '.' !in engineApi
        val serialization = reference.startsWith("io.ktor.serialization.kotlinx.json.")
        val commonKtor = client || commonEngineApi || serialization
        if (kind == Kind.DI && commonKtor) return true
        val androidType = reference.startsWith("android.") || reference.startsWith("androidx.")
        if (kind != Kind.UTIL || !androidType) return false
        if (sourceSet == "commonMain") return false
        val platforms = sources.context.compilations
            .filterNot { compilation -> compilation.test }
            .filter { compilation -> sourceSet in compilation.sourceSets }
            .map { compilation -> compilation.name.substringBefore(':') }
        return platforms.isNotEmpty() && platforms.all { platform -> platform == "androidJvm" }
    }

    private fun isAllowedCompose(sources: UnitSources, file: KoFileDeclaration, importedName: String): Boolean {
        val isUiKind = sources.placementOf(file)?.kind in COMPOSE_KINDS
        return sources.isComposeFile(file) == true && isUiKind && UnitSources.isComposeImport(importedName)
    }

    private fun isFrameworkExceptionBase(
        sources: UnitSources,
        file: KoFileDeclaration,
        reference: String
    ): Boolean {
        if (sources.kind in PLATFORM_FREE_UNITS || sources.placementOf(file)?.kind != Kind.MODEL) return false
        val name = reference.substringAfterLast('.')
        if (!EXCEPTION_NAME.matches(name)) return false
        val importDeclaration = file.imports.firstOrNull { declaration -> declaration.name == reference }
        val alias = importDeclaration?.alias?.name ?: name
        return file.topLevelClasses().any { declaration ->
            declaration.parents().any { parent -> parent.name == reference || parent.name == alias }
        }
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles
            .filter { file -> isChecked(sources, file) }
            .flatMap { file ->
                file.sourceReferences()
                    .filter { importedName -> PLATFORM_PREFIXES.any(importedName::startsWith) }
                    .filterNot { importedName ->
                        isAllowedCompose(
                            sources,
                            file,
                            importedName
                        )
                    }
                    .filterNot { reference ->
                        isAllowedCoreIntegration(
                            sources,
                            file,
                            reference
                        )
                    }
                    .filterNot { reference ->
                        isFrameworkExceptionBase(
                            sources,
                            file,
                            reference
                        )
                    }
                    .map { importedName ->
                        val message = "$importedName is a platform type: keep it in a platform implementation or a " +
                            "technology package and map it to the model at their edge. " +
                            "Explicit api, model and usecase packages remain platform-free in every unit role."
                        sources.createViolation(
                            RULE,
                            file,
                            importedName,
                            message
                        )
                    }
            }
    }

    private companion object {
        const val RULE = "PlatformImport"

        @JvmField
        val COMPOSE_KINDS = setOf(Kind.MODEL, Kind.ENTRY_POINT)

        @JvmField
        val PLATFORM_FREE_UNITS = setOf(
            UnitKind.API,
            UnitKind.CORE_API,
            UnitKind.CORE,
            UnitKind.STORAGE_CONTRACT,
            UnitKind.CONTRACT
        )

        @JvmField
        val EXCEPTION_NAME = Regex("""\w*(Exception|Error|Throwable)""")

        @JvmField
        val CHECKED_KINDS = setOf(
            Kind.MODEL,
            Kind.CONTRACT,
            Kind.USECASE
        )

        @JvmField
        val PLATFORM_PREFIXES = listOf(
            "android.",
            "androidx.",
            "com.velocitypowered.",
            "io.ktor.",
            "io.papermc.",
            "net.fabricmc.",
            "net.md_5.",
            "net.minecraft.",
            "net.minecraftforge.",
            "net.neoforged.",
            "org.bukkit.",
            "org.jetbrains.exposed.",
            "org.jetbrains.compose.",
            "org.spigotmc.",
        )
    }
}
