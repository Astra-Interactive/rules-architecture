package ru.astrainteractive.klibs.rules.gradle.detekt.extension.model

import org.gradle.api.file.ConfigurableFileCollection
import org.gradle.api.provider.Property

public abstract class DetektRulesExtension {
    /**
     * Falls back to the Kotlin JVM target of the project, then to the JVM that runs Gradle.
     */
    public abstract val jvmTarget: Property<String>

    /**
     * Configs read after the bundled one and before the project's own detekt config, so the project's settings win.
     */
    public abstract val ruleConfigs: ConfigurableFileCollection
}
