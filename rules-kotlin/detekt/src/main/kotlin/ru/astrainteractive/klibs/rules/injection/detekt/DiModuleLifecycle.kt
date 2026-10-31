package ru.astrainteractive.klibs.rules.injection.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtCallExpression
import org.jetbrains.kotlin.psi.KtClassOrObject
import org.jetbrains.kotlin.psi.KtNamedFunction
import org.jetbrains.kotlin.psi.KtProperty
import org.jetbrains.kotlin.psi.psiUtil.collectDescendantsOfType
import org.jetbrains.kotlin.psi.psiUtil.containingClassOrObject

/**
 * Reports a DI module (a class, interface or object named `…Module`) that declares its own `enable`, `disable`,
 * `reload`, `onEnable`, `onDisable` or `onReload` instead of exposing a `Lifecycle`, and a DI module that starts
 * something, a warm-up, a migration, a listener, a task, a service or a `launch`, without a `lifecycle` property.
 */
internal class DiModuleLifecycle(config: Config) : Rule(config, DESCRIPTION) {

    private fun findStartedName(module: KtClassOrObject): String? {
        return module.collectDescendantsOfType<KtCallExpression>()
            .mapNotNull { call -> call.calleeExpression?.text }
            .firstOrNull { name -> name == LAUNCH || STARTED_SUFFIXES.any(name::endsWith) }
    }

    override fun visitClassOrObject(classOrObject: KtClassOrObject) {
        super.visitClassOrObject(classOrObject)
        val name = classOrObject.name ?: return
        if (!name.endsWith(MODULE_SUFFIX)) return
        val hasLifecycle = classOrObject.declarations
            .filterIsInstance<KtProperty>()
            .any { property -> property.name == LIFECYCLE }
        if (hasLifecycle) return
        val started = findStartedName(classOrObject) ?: return
        val message = "Expose val lifecycle: Lifecycle from $name: it starts $started, and the parent module " +
            "starts and stops it through that lifecycle."
        report(Finding(Entity.atName(classOrObject), message))
    }

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
        const val LIFECYCLE = "lifecycle"
        const val LAUNCH = "launch"

        @JvmField
        val STARTED_SUFFIXES = listOf(
            "WarmUp",
            "Migration",
            "Listener",
            "Task",
            "Service"
        )

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
