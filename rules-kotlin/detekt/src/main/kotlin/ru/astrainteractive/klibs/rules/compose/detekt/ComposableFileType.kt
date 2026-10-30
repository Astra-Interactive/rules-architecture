package ru.astrainteractive.klibs.rules.compose.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtFile
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType
import ru.astrainteractive.klibs.rules.detekt.util.isComponentDefaults

/**
 * Reports a top-level class, interface or object in a file that holds a `@Composable` function. The public
 * `<Component>Defaults` object of a design-system component is the one declaration allowed beside composables.
 */
internal class ComposableFileType(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitKtFile(file: KtFile) {
        super.visitKtFile(file)
        val hasComposable = file.collectDescendantsOfType<KtNamedFunction>()
            .any { function -> function.hasAnnotation(COMPOSABLE) }
        if (!hasComposable) return
        file.declarations
            .filterIsInstance<KtClassOrObject>()
            .filterNot(KtClassOrObject::isComponentDefaults)
            .forEach { declaration ->
                val message = "Move '${declaration.name}' out of this composable file: a composable file declares " +
                    "no class; a data class goes to model/, shared values to theme/."
                report(Finding(Entity.atName(declaration), message))
            }
    }

    private companion object {
        const val COMPOSABLE = "Composable"
        const val DESCRIPTION = "A composable file declares no class, except a design-system <Component>Defaults."
    }
}
