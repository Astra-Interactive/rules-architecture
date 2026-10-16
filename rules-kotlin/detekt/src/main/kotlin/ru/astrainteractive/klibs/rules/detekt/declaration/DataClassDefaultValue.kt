package ru.astrainteractive.klibs.rules.detekt.declaration

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.psi.KtClass

/**
 * Reports a default value on a `data class` constructor parameter.
 * A `@Serializable` data class is exempt, because serialization needs the defaults.
 */
internal class DataClassDefaultValue(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitClass(klass: KtClass) {
        super.visitClass(klass)
        if (!klass.isData()) return
        if (klass.hasAnnotation(SERIALIZABLE)) return
        klass.primaryConstructorParameters
            .filter { parameter -> parameter.hasDefaultValue() }
            .forEach { parameter ->
                val message = "Remove the default of '${parameter.name}' and pass the value from the caller."
                report(Finding(Entity.from(parameter), message))
            }
    }

    private companion object {
        const val SERIALIZABLE = "Serializable"
        const val DESCRIPTION = "A data class that is not @Serializable has no default constructor values."
    }
}
