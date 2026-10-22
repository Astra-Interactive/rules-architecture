package ru.astrainteractive.klibs.rules.detekt.compose

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.name.Name
import org.jetbrains.kotlin.psi.KtFile
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType

/**
 * Reports a file in a `composable` package that declares no `@Composable` function: a file of plain functions
 * (`DrawScope` extensions, a brush builder) is a `util` file.
 */
internal class NonComposableFileInComposable(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitKtFile(file: KtFile) {
        super.visitKtFile(file)
        val segments = file.packageFqName.pathSegments().map(Name::asString)
        if (COMPOSABLE_SEGMENT !in segments) return
        val first = file.declarations.firstOrNull() ?: return
        val hasComposable = file.collectDescendantsOfType<KtNamedFunction>()
            .any { function -> function.hasAnnotation(COMPOSABLE) }
        if (hasComposable) return
        val message = "Move ${file.name} out of the composable package: a file with no @Composable function is a " +
            "util file, util/<Subject>Ext.kt for extensions or a file named after its function."
        report(Finding(Entity.from(first), message))
    }

    private companion object {
        const val COMPOSABLE = "Composable"
        const val COMPOSABLE_SEGMENT = "composable"
        const val DESCRIPTION = "A file in a composable package declares a @Composable function."
    }
}
