package ru.astrainteractive.klibs.rules.konsist.check.placement.konsist

import com.lemonappdev.konsist.api.declaration.KoFileDeclaration
import ru.astrainteractive.klibs.rules.core.unit.model.UnitKind
import ru.astrainteractive.klibs.rules.konsist.check.konsist.ArchitectureCheck
import ru.astrainteractive.klibs.rules.konsist.check.konsist.UnitSources
import ru.astrainteractive.klibs.rules.konsist.check.model.Violation
import ru.astrainteractive.klibs.rules.konsist.check.model.util.packageName

/**
 * Reports a `composable` or `theme` package whose production compilation has no Compose dependency: those words
 * mean Kotlin Compose, and any other UI framework takes its words from its own classes. `viewmodel` is an
 * Android-style view model with or without Compose, so it is not judged.
 */
internal class ComposeKindWithoutCompose : ArchitectureCheck {
    override val unitKinds: Set<UnitKind> = UnitKind.entries.toSet()

    override fun check(sources: UnitSources): List<Violation> {
        return sources.mainFiles
            .distinctBy(KoFileDeclaration::packageName)
            .mapNotNull { file ->
                val word = sources.placementOf(file)?.kindWord?.takeIf { kindWord -> kindWord in COMPOSE_WORDS }
                    ?: return@mapNotNull null
                val compose = requireNotNull(sources.isComposeFile(file)) {
                    "Compilation context is missing for ${file.path}; pass --unit-context from the Gradle linter"
                }
                if (compose) return@mapNotNull null
                val message = "Rename the package '$word': it means Kotlin Compose, and this unit does not use " +
                    "Compose; name the kind after the UI framework's own classes."
                sources.createViolation(
                    RULE,
                    file,
                    file.packageName,
                    message
                )
            }
    }

    private companion object {
        const val RULE = "ComposeKindWithoutCompose"

        @JvmField
        val COMPOSE_WORDS = setOf("composable", "theme")
    }
}
