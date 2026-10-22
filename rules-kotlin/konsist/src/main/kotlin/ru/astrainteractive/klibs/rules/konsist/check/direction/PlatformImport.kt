package ru.astrainteractive.klibs.rules.konsist.check.direction

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import com.lemonappdev.konsist.api.declaration.KoImportDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.importedName
import ru.astrainteractive.klibs.rules.konsist.layout.Kind
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a platform, framework or ORM import in `model`, `api` and `usecase`, and anywhere in an `api` unit,
 * which holds platform-free code only. A technology unit is not checked: it is that technology's
 * implementation, its own `model` included. The `model` of a Compose UI unit may import Compose.
 */
internal class PlatformImport : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet() - UnitKind.TECHNOLOGY

    private fun isChecked(sources: UnitSources, file: KoFileDeclaration): Boolean {
        if (sources.kind == UnitKind.API) return true
        return sources.placementOf(file)?.kind in CHECKED_KINDS
    }

    private fun isAllowedCompose(sources: UnitSources, file: KoFileDeclaration, importedName: String): Boolean {
        val isUiModel = sources.isComposeUnit && sources.placementOf(file)?.kind == Kind.MODEL
        return isUiModel && UnitSources.isComposeImport(importedName)
    }

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles
            .filter { file -> isChecked(sources, file) }
            .flatMap { file ->
                file.imports
                    .map(KoImportDeclaration::importedName)
                    .filter { importedName -> PLATFORM_PREFIXES.any(importedName::startsWith) }
                    .filterNot { importedName ->
                        isAllowedCompose(
                            sources,
                            file,
                            importedName
                        )
                    }
                    .map { importedName ->
                        val message = "$importedName is a platform type: keep it in a platform unit or a " +
                            "technology package and map it to the model at their edge."
                        Violation(
                            RULE,
                            sources.relativePath(file),
                            importedName,
                            message
                        )
                    }
            }
    }

    private companion object {
        const val RULE = "PlatformImport"

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
        )
    }
}
