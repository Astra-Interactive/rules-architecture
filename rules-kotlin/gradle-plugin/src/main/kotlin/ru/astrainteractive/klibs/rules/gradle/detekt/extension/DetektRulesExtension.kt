package ru.astrainteractive.klibs.rules.gradle.detekt.extension

import org.gradle.api.provider.Property

/**
 * Settings of the detekt plugin.
 */
public abstract class DetektRulesExtension {
    /**
     * The JVM target detekt resolves types against, such as `17`. Taken from the Kotlin JVM target of the project
     * when it has one, and from the JVM that runs Gradle otherwise.
     */
    public abstract val jvmTarget: Property<String>
}
