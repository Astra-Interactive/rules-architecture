package ru.astrainteractive.klibs.rules.detekt.compose

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.psi.KtDeclaration
import org.jetbrains.kotlin.psi.KtFile
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtProperty

/**
 * Checks that the previews close a composable file: after the first `@Preview` function only preview helpers
 * follow (functions with `Preview` in the name, values named `PREVIEW_…` or `Preview…`), and every `@Preview`
 * function is named `…Preview`.
 */
internal class PreviewBlockLast(config: Config) : Rule(config, DESCRIPTION) {

    private fun isPreview(declaration: KtDeclaration): Boolean {
        return declaration is KtNamedFunction && declaration.hasAnnotation(PREVIEW)
    }

    private fun findMessageAfterPreviews(declaration: KtDeclaration): String? {
        val name = declaration.name.orEmpty()
        val isPreviewFunction = declaration is KtNamedFunction && PREVIEW in name
        val hasPreviewPrefix = PREVIEW_VALUE_PREFIXES.any(name::startsWith)
        val isPreviewValue = declaration is KtProperty && hasPreviewPrefix
        val isOther = declaration !is KtNamedFunction && declaration !is KtProperty
        if (isPreviewFunction || isPreviewValue || isOther || isPreview(declaration)) return null
        return "Move '$name' above the previews, or name it for the preview block: the previews close the file " +
            "and hold only preview functions and PREVIEW_/Preview values."
    }

    override fun visitKtFile(file: KtFile) {
        super.visitKtFile(file)
        val declarations = file.declarations
        declarations
            .filter(::isPreview)
            .filterNot { function -> function.name.orEmpty().endsWith(PREVIEW) }
            .forEach { function ->
                val message = "Name the preview '${function.name}' <Composable><Scenario>Preview."
                report(Finding(Entity.from(function), message))
            }
        val firstPreview = declarations.indexOfFirst(::isPreview)
        if (firstPreview < 0) return
        declarations.drop(firstPreview + 1).forEach { declaration ->
            val message = findMessageAfterPreviews(declaration) ?: return@forEach
            report(Finding(Entity.from(declaration), message))
        }
    }

    private companion object {
        const val PREVIEW = "Preview"

        @JvmField
        val PREVIEW_VALUE_PREFIXES = listOf("PREVIEW_", "Preview")
        const val DESCRIPTION = "Previews close a composable file and everything after the first one serves them."
    }
}
