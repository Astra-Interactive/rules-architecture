package ru.astrainteractive.klibs.rules.detekt.naming

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import org.jetbrains.kotlin.psi.KtClass
import org.jetbrains.kotlin.psi.KtClassOrObject

/**
 * Reports the `Impl` and `Interface` suffixes on any type and the `I` prefix on an interface (`IRepository`).
 */
internal class ImplementationName(config: Config) : Rule(config, DESCRIPTION) {

    private fun findViolations(declaration: KtClassOrObject, name: String): List<String> {
        return buildList {
            if (name.endsWith(IMPL_SUFFIX)) {
                add("Name '$name' by its technology or variant instead of the '$IMPL_SUFFIX' suffix.")
            }
            if (name.endsWith(INTERFACE_SUFFIX)) {
                add("Remove the '$INTERFACE_SUFFIX' suffix from '$name'.")
            }
            if (declaration is KtClass && declaration.isInterface() && INTERFACE_PREFIX.matches(name)) {
                add("Remove the 'I' prefix from '$name'.")
            }
        }
    }

    override fun visitClassOrObject(classOrObject: KtClassOrObject) {
        super.visitClassOrObject(classOrObject)
        val name = classOrObject.name ?: return
        findViolations(classOrObject, name).forEach { message ->
            report(Finding(Entity.from(classOrObject), message))
        }
    }

    private companion object {
        const val IMPL_SUFFIX = "Impl"
        const val INTERFACE_SUFFIX = "Interface"

        @JvmField
        val INTERFACE_PREFIX = Regex("I[A-Z][a-z].*")
        const val DESCRIPTION = "An interface takes the plain name; an implementation is named by its technology."
    }
}
