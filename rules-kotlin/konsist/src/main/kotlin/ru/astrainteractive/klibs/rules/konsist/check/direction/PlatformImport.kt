package ru.astrainteractive.klibs.rules.konsist.check.direction

import ru.astrainteractive.klibs.rules.konsist.check.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.model.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.importedName
import ru.astrainteractive.klibs.rules.konsist.layout.Layer
import ru.astrainteractive.klibs.rules.konsist.layout.UnitKind

/**
 * Reports a platform or framework import in a use case or a contract, which reach the server only through the
 * library's abstractions or the context's own `integration` contract. `model` is checked by detekt's
 * `ForbiddenImport/model`.
 */
internal class PlatformImport : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = setOf(UnitKind.CONTEXT)

    override fun check(sources: UnitSources): List<Violation> = sources.mainFiles
        .filter { file -> sources.placementOf(file)?.layer in CHECKED_LAYERS }
        .flatMap { file ->
            file.imports
                .map { import -> import.importedName }
                .filter { importedName -> PLATFORM_PREFIXES.any { prefix -> importedName.startsWith(prefix) } }
                .map { importedName ->
                    val message = "$importedName is a platform type: reach it through an integration contract."
                    Violation(RULE, sources.relativePath(file), importedName, message)
                }
        }

    private companion object {
        const val RULE = "PlatformImport"

        @JvmField
        val CHECKED_LAYERS = setOf(Layer.USE_CASE, Layer.CONTRACT)

        @JvmField
        val PLATFORM_PREFIXES = listOf(
            "android.",
            "androidx.",
            "io.ktor.",
            "io.papermc.",
            "net.fabricmc.",
            "net.minecraft.",
            "net.minecraftforge.",
            "net.neoforged.",
            "org.bukkit.",
            "org.jetbrains.exposed.",
        )
    }
}
