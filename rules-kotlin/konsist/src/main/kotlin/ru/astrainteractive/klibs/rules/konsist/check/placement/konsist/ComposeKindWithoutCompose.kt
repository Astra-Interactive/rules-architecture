package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.packageName
import ru.astrainteractive.klibs.rules.konsist.layout.model.UnitKind

/**
 * Reports a `composable`, `viewmodel` or `theme` package in a unit whose main sources import nothing of Compose:
 * those words mean Kotlin Compose, and any other UI framework takes its words from its own classes.
 */
internal class ComposeKindWithoutCompose : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    override fun check(sources: UnitSources): List<Violation> {
        val usesCompose = sources.mainFiles
            .flatMap(KoFileDeclaration::imports)
            .any { import -> COMPOSE_PREFIXES.any { prefix -> import.name.startsWith(prefix) } }
        if (usesCompose) return emptyList()
        return sources.mainFiles
            .distinctBy(KoFileDeclaration::packageName)
            .mapNotNull { file ->
                val word = file.packageName.split('.').firstOrNull { segment -> segment in COMPOSE_WORDS }
                    ?: return@mapNotNull null
                val message = "Rename the package '$word': it means Kotlin Compose, and this unit does not use " +
                    "Compose; name the kind after the UI framework's own classes."
                Violation(
                    RULE,
                    sources.relativePath(file),
                    file.packageName,
                    message
                )
            }
    }

    private companion object {
        const val RULE = "ComposeKindWithoutCompose"

        @JvmField
        val COMPOSE_WORDS = setOf(
            "composable",
            "viewmodel",
            "theme"
        )

        @JvmField
        val COMPOSE_PREFIXES = listOf("androidx.compose.", "org.jetbrains.compose.")
    }
}
