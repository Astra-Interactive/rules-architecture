package ru.astrainteractive.klibs.rules.declaration.detekt

import dev.detekt.api.Config
import dev.detekt.api.Entity
import dev.detekt.api.Finding
import dev.detekt.api.Rule
import dev.detekt.psi.hasAnnotation
import org.jetbrains.kotlin.psi.KtClass
import org.jetbrains.kotlin.psi.KtNamedDeclaration
import org.jetbrains.kotlin.psi.KtParameter

/**
 * Reports a serialized property of a `@Serializable` class that carries no `@SerialName`.
 * A `@Transient` property is not serialized and is exempt; an enum entry cannot carry the annotation.
 */
internal class SerialNameNotProvided(config: Config) : Rule(config, DESCRIPTION) {

    override fun visitClass(klass: KtClass) {
        super.visitClass(klass)
        if (!klass.hasAnnotation(SERIALIZABLE)) return
        klass.primaryConstructorParameters
            .filter(KtParameter::hasValOrVar)
            .forEach(::reportMissingSerialName)
        klass.getBody()
            ?.properties
            .orEmpty()
            .forEach(::reportMissingSerialName)
    }

    private fun reportMissingSerialName(property: KtNamedDeclaration) {
        if (property.hasAnnotation(TRANSIENT)) return
        if (property.hasAnnotation(SERIAL_NAME)) return
        val message = "Annotate '${property.name}' with @SerialName."
        report(Finding(Entity.from(property), message))
    }

    private companion object {
        const val DESCRIPTION = "A serialized property of a @Serializable class carries @SerialName."
        const val SERIALIZABLE = "Serializable"
        const val SERIAL_NAME = "SerialName"
        const val TRANSIENT = "Transient"
    }
}
