package ru.astrainteractive.klibs.rules.detekt.naming

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.name.Name
import org.jetbrains.kotlin.psi.KtClassOrObject

/**
 * Reports a type name the domain rule forbids or its kind places: the `Validator` and `Factory` suffixes
 * anywhere, and `WarmUp`, `Migration` and `ViewModel` outside the `warmup`, `migration` and `viewmodel` packages.
 */
internal class RolelessClassName(config: Config) : Rule(config, DESCRIPTION) {

    private fun findMessage(name: String, segments: List<String>): String? {
        if (name.endsWith(VALIDATOR)) {
            return "Remove '$name': a validator duplicates a model invariant; validate the input at the entry " +
                "point, guard the invariant in the value object or the aggregate's rules in util."
        }
        if (name.endsWith(FACTORY)) {
            return "Remove '$name': dependencies are constructed in di/, and a factory with a rule is a top-level " +
                "create<Aggregate>() in util."
        }
        val kind = KIND_BY_SUFFIX.entries.firstOrNull { entry -> name.endsWith(entry.key) } ?: return null
        if (kind.value in segments) return null
        return "Move '$name' to a '${kind.value}' package: the ${kind.key} suffix names that kind."
    }

    override fun visitClassOrObject(classOrObject: KtClassOrObject) {
        super.visitClassOrObject(classOrObject)
        val name = classOrObject.name ?: return
        val segments = classOrObject.containingKtFile.packageFqName.pathSegments().map(Name::asString)
        val message = findMessage(name, segments) ?: return
        report(Finding(Entity.atName(classOrObject), message))
    }

    private companion object {
        const val VALIDATOR = "Validator"
        const val FACTORY = "Factory"

        @JvmField
        val KIND_BY_SUFFIX = mapOf(
            "WarmUp" to "warmup",
            "Migration" to "migration",
            "ViewModel" to "viewmodel"
        )
        const val DESCRIPTION = "No Validator or Factory classes; WarmUp, Migration and ViewModel sit in their kinds."
    }
}
