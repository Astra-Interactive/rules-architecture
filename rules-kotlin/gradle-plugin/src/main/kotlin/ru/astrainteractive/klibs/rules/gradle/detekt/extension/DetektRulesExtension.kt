package ru.astrainteractive.klibs.rules.gradle.detekt.extension

import org.gradle.api.provider.Property

public abstract class DetektRulesExtension {
    /**
     * Falls back to the Kotlin JVM target of the project, then to the JVM that runs Gradle.
     */
    public abstract val jvmTarget: Property<String>
}
