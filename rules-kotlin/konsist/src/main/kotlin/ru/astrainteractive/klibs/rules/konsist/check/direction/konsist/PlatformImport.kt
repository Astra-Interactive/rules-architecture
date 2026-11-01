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
 * which holds platform-free code only. A technology unit is not checked: it is that technology's
 * implementation, its own `model` included. In a Compose UI unit, `model` and the UI kinds may import Compose: an
 * `api` unit whose state holder declares `Render()` is such a unit.
 */
internal class PlatformImport : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet() - UnitKind.TECHNOLOGY - UnitKind.PLATFORM

    private fun isChecked(sources: UnitSources, file: KoFileDeclaration): Boolean {
        if (sources.kind == UnitKind.API || sources.kind == UnitKind.CORE_API) return true
        return sources.placementOf(file)?.kind in CHECKED_KINDS
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
                        isFrameworkExceptionBase(
                            sources,
                            file,
                            reference
                        )
                    }
                    .map { importedName ->
                        val message = "$importedName is a platform type: keep it in a platform unit or a " +
                            "technology package and map it to the model at their edge."
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
