package ru.astrainteractive.klibs.rules.injection.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.psiUtil.containingClassOrObject

/**
 * Reports a DI module (a class, interface or object named `…Module`) that declares its own `enable`, `disable`,
 * `reload`, `onEnable`, `onDisable` or `onReload` instead of exposing a `Lifecycle`.
 */
internal class DiModuleLifecycle(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitNamedFunction(function: KtNamedFunction) {
        super.visitNamedFunction(function)
        val owner = function.containingClassOrObject?.name ?: return
        if (!owner.endsWith(MODULE_SUFFIX) || function.name !in LIFECYCLE_NAMES) return
        val message = "Remove '${function.name}' from $owner: expose val lifecycle: Lifecycle and let the parent " +
            "module collect it."
        report(Finding(Entity.atName(function), message))
    }

    private companion object {
        const val MODULE_SUFFIX = "Module"

        @JvmField
        val LIFECYCLE_NAMES = setOf(
            "enable",
            "disable",
            "reload",
            "onEnable",
            "onDisable",
            "onReload"
        )
        const val DESCRIPTION = "A DI module exposes val lifecycle: Lifecycle, never its own enable/disable/reload."
    }
}
